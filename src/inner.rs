use std::collections::HashMap;
use std::hash::Hash;

use left_right::{
    Absorb,
    aliasing::{Aliased, DropBehavior},
};

use crate::mutable::Mutable;

pub(crate) struct Inner<Key, MutV, RefV, Meta, D = crate::aliasing::NoDrop>
where
    D: DropBehavior,
    MutV: Clone,
    Meta: Clone,
{
    pub(crate) data: HashMap<Key, Value<MutV, RefV, D>>,
    pub(crate) meta: Meta,
    pub(crate) ready: bool,
}

pub struct Value<MutV, RefV, D>
where
    D: DropBehavior,
    MutV: Clone,
{
    /// Can be mutated whilst in the map but has to be allocated twice
    pub(crate) mut_v: MutV,
    /// Cannot be mutated whilst in the map; but is allocated only once
    pub(crate) ref_v: Aliased<RefV, D>,
}

impl<MutV, RefV> Value<MutV, RefV, crate::aliasing::NoDrop>
where
    MutV: Clone,
{
    /// Read the mutable state of this value
    /// (An operation is required to mutate this state)
    pub fn mut_v(&self) -> &MutV {
        &self.mut_v
    }

    pub fn ref_v(&self) -> &RefV {
        self.ref_v.as_ref()
    }

    /// Produce a copy of this Value by:
    /// - Aliasing the immutable part (RefV)
    /// - Cloning the mutable part (MutV)
    ///
    /// # Safety
    /// See left_right::Aliased::alias. You must make sure
    /// you get the drop order correct.
    pub(crate) unsafe fn alias_clone(&self) -> Self {
        Value {
            mut_v: self.mut_v.clone(),
            // SAFETY:
            // - No &mut self.ref_v is ever given out
            // - Our implemenation of Absorb ensures proper
            //   drop order.
            ref_v: unsafe { self.ref_v.alias() },
        }
    }

    /// Change the drop behaviour of the immutable RefV part
    /// of this value.
    ///
    /// # Safety
    ///
    /// It is always safe to change an `Aliased` from a dropping `D` to a non-dropping `D`. Going
    /// the other way around is only safe if `self` is the last alias for the `T`.
    pub(crate) unsafe fn change_drop<D2: DropBehavior>(self) -> Value<MutV, RefV, D2> {
        Value {
            mut_v: self.mut_v,
            ref_v: unsafe { self.ref_v.change_drop() },
        }
    }
}

pub(crate) enum Operation<Key, MutV, RefV, Op>
where
    MutV: Clone,
{
    Insert(Key, Value<MutV, RefV, crate::aliasing::NoDrop>),
    Remove(Key),
    Clear,

    /// Mark the map as ready to be consumed for readers.
    MarkReady,

    Mutate(Key, Op),
}

impl<Key, MutV, RefV, Meta, Op> Absorb<Operation<Key, MutV, RefV, Op>>
    for Inner<Key, MutV, RefV, Meta>
where
    Key: Eq + Hash + Clone,
    MutV: Mutable<Op> + Clone,
    Meta: Clone,
{
    fn absorb_first(&mut self, op: &mut Operation<Key, MutV, RefV, Op>, _other: &Self) {
        // Safety note for calls to .alias():
        //
        //   it is safe to alias this value here because if it is ever removed, one alias is always
        //   first dropped with NoDrop (in absorb_first), and _then_ the other (and only remaining)
        //   alias is dropped with DoDrop (in absorb_second). we won't drop the aliased value until
        //   _after_ absorb_second is called on this operation, so leaving an alias in the oplog is
        //   also safe.

        match *op {
            Operation::Insert(ref key, ref mut value) => {
                self.data
                    .insert(key.clone(), unsafe { value.alias_clone() });
            }
            Operation::Remove(ref key) => {
                self.data.remove(key);
            }
            Operation::Clear => {
                self.data.clear();
            }
            Operation::MarkReady => {
                self.ready = true;
            }
            Operation::Mutate(ref key, ref mut operation) => {
                if let Some(value) = self.data.get_mut(key) {
                    let mut_v = &mut value.mut_v;

                    // SAFETY:
                    // The implementor of GetMut::mutate must ensure
                    // the function is safe
                    unsafe {
                        Mutable::mutate(mut_v, operation);
                    }
                }
            }
        }
    }

    fn absorb_second(&mut self, op: Operation<Key, MutV, RefV, Op>, _other: &Self) {
        // # Safety (for cast):
        //
        // See the module-level documentation for left_right::aliasing.
        // NoDrop and DoDrop are both private, therefore this cast is (likely) sound.
        //
        // # Safety (for NoDrop -> DoDrop):
        //
        // It is safe for us to drop values the second time each operation has been
        // performed, since if they are dropped here, they were also dropped in the first
        // application of the operation, which removed the only other alias.
        let inner: &mut Inner<Key, MutV, RefV, Meta, crate::aliasing::DoDrop> =
            unsafe { &mut *(self as *mut _ as *mut _) };

        // Safety note for calls to .change_drop():
        //
        //   we're turning a NoDrop into DoDrop, so we must be prepared for a drop.
        //   if absorb_first dropped its alias, then `value` is the only alias
        //   if absorb_first did not drop its alias, then `value` will not be dropped here either,
        //   and at the end of scope we revert to `NoDrop`, so all is well.
        match op {
            Operation::Insert(key, value) => {
                inner
                    .data
                    .insert(key.clone(), unsafe { value.change_drop() });
            }
            Operation::Remove(key) => {
                inner.data.remove(&key);
            }
            Operation::Clear => {
                inner.data.clear();
            }
            Operation::MarkReady => {
                inner.ready = true;
            }
            Operation::Mutate(key, mut operation) => {
                if let Some(value) = self.data.get_mut(&key) {
                    let mut_v = &mut value.mut_v;

                    // SAFETY:
                    // The implementor of GetMut::mutate must ensure
                    // the function is safe
                    unsafe {
                        Mutable::mutate(mut_v, &mut operation);
                    }
                }
            }
        }
    }

    fn drop_first(self: Box<Self>) {
        // since the two copies are exactly equal, we need to make sure that we *don't* call the
        // destructors of any of the values that are in our map, as they'll all be called when the
        // last read handle goes out of scope. that's easy enough since none of them will be
        // dropped by default.
    }

    fn drop_second(self: Box<Self>) {
        // when the second copy is dropped is where we want to _actually_ drop all the values in
        // the map. we do this by setting the generic type to the one that causes drops to happen.
        //
        // safety: since we're going second, we know that all the aliases in the first map have
        // gone away, so all of our aliases must be the only ones.
        let inner: Box<Inner<Key, MutV, RefV, Meta, crate::aliasing::DoDrop>> =
            unsafe { Box::from_raw(Box::into_raw(self) as *mut _ as *mut _) };
        drop(inner);
    }

    fn sync_with(&mut self, first: &Self) {
        let inner: &mut Inner<Key, MutV, RefV, Meta, crate::aliasing::DoDrop> =
            unsafe { &mut *(self as *mut _ as *mut _) };
        inner.data.extend(first.data.iter().map(|(k, vs)| {
            // # Safety (for aliasing):
            //
            // We are aliasing every value in the read map, and the oplog has no other
            // pending operations (by the semantics of JustCloneRHandle). For any of the
            // values we alias to be dropped, the operation that drops it must first be
            // enqueued to the oplog, at which point it will _first_ go through
            // absorb_first, which will remove the alias and leave only one alias left.
            // Only after that, when that operation eventually goes through absorb_second,
            // will the alias be dropped, and by that time it is the only value.
            //
            // # Safety (for hashing):
            //
            // Due to `RandomState` there can be subtle differences between the iteration order
            // of two `HashMap` instances. We prevent this by using `left_right::new_with_empty`,
            // which `clone`s the first map, making them use the same hasher.
            //
            // # Safety (for NoDrop -> DoDrop):
            //
            // The oplog has only this one operation in it for the first call to `publish`,
            // so we are about to turn the alias back into NoDrop.
            (k.clone(), unsafe { vs.alias_clone().change_drop() })
        }));
        self.ready = true;
    }
}

impl<Key, MutV, RefV, Meta> Clone for Inner<Key, MutV, RefV, Meta>
where
    MutV: Clone,
    Meta: Clone,
{
    fn clone(&self) -> Self {
        assert!(self.data.is_empty());
        Self {
            data: HashMap::with_capacity_and_hasher(
                self.data.capacity(),
                self.data.hasher().clone(),
            ),
            meta: self.meta.clone(),
            ready: self.ready,
        }
    }
}

impl<Key, MutV, RefV, Meta> Inner<Key, MutV, RefV, Meta>
where
    Key: Eq + Hash,
    MutV: Clone,
    Meta: Clone,
{
    pub(crate) fn with_capacity(meta: Meta, capacity: usize) -> Self {
        Inner {
            data: HashMap::with_capacity(capacity),
            meta,
            ready: false,
        }
    }

    pub(crate) fn new(meta: Meta) -> Self {
        Inner {
            data: HashMap::new(),
            meta,
            ready: false,
        }
    }
}
