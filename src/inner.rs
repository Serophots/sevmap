use std::collections::HashMap;
use std::hash::Hash;

use left_right::{
    Absorb,
    aliasing::{Aliased, DropBehavior},
};

pub(crate) struct Inner<K, V, M, D = crate::aliasing::NoDrop>
where
    D: DropBehavior,
    M: Clone,
{
    pub(crate) data: HashMap<K, Aliased<V, D>>,
    pub(crate) meta: M,
    pub(crate) ready: bool,
}

pub(crate) enum Operation<K, V> {
    Insert(K, Aliased<V, crate::aliasing::NoDrop>),
    Remove(K),
    Clear,

    /// Mark the map as ready to be consumed for readers.
    MarkReady,
}

impl<K, V, M> Absorb<Operation<K, V>> for Inner<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    fn absorb_first(&mut self, op: &mut Operation<K, V>, _other: &Self) {
        // Safety note for calls to .alias():
        //
        //   it is safe to alias this value here because if it is ever removed, one alias is always
        //   first dropped with NoDrop (in absorb_first), and _then_ the other (and only remaining)
        //   alias is dropped with DoDrop (in absorb_second). we won't drop the aliased value until
        //   _after_ absorb_second is called on this operation, so leaving an alias in the oplog is
        //   also safe.

        match *op {
            Operation::Insert(ref key, ref mut value) => {
                self.data.insert(key.clone(), unsafe { value.alias() });
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
        }
    }

    fn absorb_second(&mut self, op: Operation<K, V>, _other: &Self) {
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
        let inner: &mut Inner<K, V, M, crate::aliasing::DoDrop> =
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
        let inner: Box<Inner<K, V, M, crate::aliasing::DoDrop>> =
            unsafe { Box::from_raw(Box::into_raw(self) as *mut _ as *mut _) };
        drop(inner);
    }

    fn sync_with(&mut self, first: &Self) {
        let inner: &mut Inner<K, V, M, crate::aliasing::DoDrop> =
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
            (k.clone(), unsafe { vs.alias().change_drop() })
        }));
        self.ready = true;
    }
}

impl<K, V, M> Clone for Inner<K, V, M>
where
    M: Clone,
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

impl<K, V, M> Inner<K, V, M>
where
    K: Eq + Hash,
    M: Clone,
{
    pub(crate) fn with_capacity(meta: M, capacity: usize) -> Self {
        Inner {
            data: HashMap::with_capacity(capacity),
            meta,
            ready: false,
        }
    }

    pub(crate) fn new(meta: M) -> Self {
        Inner {
            data: HashMap::new(),
            meta,
            ready: false,
        }
    }
}
