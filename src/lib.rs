// This _should_ detect if we ever accidentally leak aliasing::NoDrop.
// But, currently, it does not..
#![deny(unreachable_pub)]

use handles::single::ReadHandle as SingleReadHandle;
use handles::single::WriteHandle as SingleWriteHandle;

use crate::inner::Inner;
use crate::inner::Operation;
use crate::stable_hash_eq::StableHashEq;

use std::hash::Hash;

mod inner;
mod multi;
mod read_ref;
mod single;
mod stable_hash_eq;

pub mod handles {
    pub mod single {
        pub use crate::single::read::ReadHandle;
        pub use crate::single::write::WriteHandle;
    }

    pub mod multi {}
}

// NOTE: It is _critical_ that this module is not public.
mod aliasing;

#[derive(Debug)]
pub struct Options<M> {
    meta: M,
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

impl<M> Options<M> {
    pub fn with_meta<M2>(self, meta: M2) -> Options<M2> {
        Options {
            meta,
            capacity: self.capacity,
        }
    }

    pub fn with_capacity(self, capacity: usize) -> Options<M> {
        Options {
            meta: self.meta,
            capacity: Some(capacity),
        }
    }

    pub fn construct_single<K, V>(self) -> (SingleWriteHandle<K, V, M>, SingleReadHandle<K, V, M>)
    where
        K: StableHashEq + Clone,
        M: Clone + 'static,
    {
        // Safety: K: StableHashEq
        unsafe { self.assert_stable_single() }
    }

    /// Create the map, and construct the read and write handles used to access it.
    ///
    /// # Safety
    ///
    /// This method is safe to call as long as the implementation of `Hash` and `Eq` for both `K`
    /// and `V` are deterministic. That is, they must always yield the same result if given the
    /// same inputs. For keys of type `K`, the result must also be consistent between different
    /// clones of the same key.
    pub unsafe fn assert_stable_single<K, V>(
        self,
    ) -> (SingleWriteHandle<K, V, M>, SingleReadHandle<K, V, M>)
    where
        K: Eq + Hash + Clone,
        M: Clone + 'static,
    {
        let inner = match self.capacity {
            Some(cap) => Inner::with_capacity(self.meta, cap),
            None => Inner::new(self.meta),
        };

        let (mut w, r) = left_right::new_from_empty(inner);
        w.append(Operation::MarkReady);

        (SingleWriteHandle::new(w), SingleReadHandle::new(r))
    }
}

pub fn new_single<K, V>() -> (SingleWriteHandle<K, V, ()>, SingleReadHandle<K, V, ()>)
where
    K: StableHashEq + Clone,
{
    Options::default().construct_single()
}

// pub fn new_multi<>
