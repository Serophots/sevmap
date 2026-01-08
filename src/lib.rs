// This _should_ detect if we ever accidentally leak aliasing::NoDrop.
// But, currently, it does not..
#![deny(unreachable_pub)]

use crate::get_mut::Mutable;
use crate::handles::ReadHandle;
use crate::handles::WriteHandle;
use crate::inner::Inner;
use crate::inner::Operation;
use crate::stable_hash_eq::StableHashEq;

use std::hash::Hash;

mod get_mut;
mod inner;
mod multi;
mod read_ref;
mod single;
mod stable_hash_eq;

pub mod handles {
    pub use crate::single::read::ReadHandle;
    pub use crate::single::write::WriteHandle;
}

pub mod refs {
    pub use crate::inner::Value;
    pub use crate::read_ref::MapReadRef;

    // Expose `ReadGuard` since it has useful methods the user will likely care about.
    #[doc(inline)]
    pub use left_right::ReadGuard;
}

// NOTE: It is _critical_ that this module is not public.
mod aliasing;

#[derive(Debug)]
pub struct Options<Meta> {
    meta: Meta,
    capacity: Option<usize>,
}

impl Default for Options<()> {
    fn default() -> Self {
        Options {
            meta: (),
            capacity: None,
        }
    }
}

impl<Meta> Options<Meta> {
    pub fn with_meta<M2>(self, meta: M2) -> Options<M2> {
        Options {
            meta,
            capacity: self.capacity,
        }
    }

    pub fn with_capacity(self, capacity: usize) -> Options<Meta> {
        Options {
            meta: self.meta,
            capacity: Some(capacity),
        }
    }

    pub fn construct<Key, MutV, RefV, Op>(
        self,
    ) -> (
        WriteHandle<Key, MutV, RefV, Meta, Op>,
        ReadHandle<Key, MutV, RefV, Meta>,
    )
    where
        Key: StableHashEq + Clone,
        MutV: Mutable<Op> + Clone,
        Meta: Clone + 'static,
    {
        // Safety: K: StableHashEq
        unsafe { self.assert_stable() }
    }

    /// Create the map, and construct the read and write handles used to access it.
    ///
    /// # Safety
    ///
    /// This method is safe to call as long as the implementation of `Hash` and `Eq` for both `K`
    /// and `V` are deterministic. That is, they must always yield the same result if given the
    /// same inputs. For keys of type `K`, the result must also be consistent between different
    /// clones of the same key.
    pub unsafe fn assert_stable<Key, MutV, RefV, Op>(
        self,
    ) -> (
        WriteHandle<Key, MutV, RefV, Meta, Op>,
        ReadHandle<Key, MutV, RefV, Meta>,
    )
    where
        Key: Eq + Hash + Clone,
        MutV: Mutable<Op> + Clone,
        Meta: Clone + 'static,
    {
        let inner = match self.capacity {
            Some(cap) => Inner::with_capacity(self.meta, cap),
            None => Inner::new(self.meta),
        };

        // Safety:
        // We must call new_from_inner so that the HashMap is cloned from left to right on initiation
        // (Two calls to HashMap::new will have subtly different hashing behaviour)
        let (mut w, r) = left_right::new_from_empty(inner);
        w.append(Operation::MarkReady);

        (WriteHandle::new(w), ReadHandle::new(r))
    }
}

pub fn new<Key, MutV, RefV, Op>() -> (
    WriteHandle<Key, MutV, RefV, (), Op>,
    ReadHandle<Key, MutV, RefV, ()>,
)
where
    Key: StableHashEq + Clone,
    MutV: Mutable<Op> + Clone,
{
    Options::default().construct()
}

// pub fn new_multi<>
