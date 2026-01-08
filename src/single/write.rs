use left_right::aliasing::Aliased;

use crate::{
    inner::{Inner, Operation},
    single::read::ReadHandle,
};
use std::{hash::Hash, ops::Deref};

/// A write handle to a single-valued map
pub struct WriteHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    write: left_right::WriteHandle<Inner<K, V, M>, Operation<K, V>>,
    read: ReadHandle<K, V, M>,
}

impl<K, V, M> WriteHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    pub(crate) fn new(write: left_right::WriteHandle<Inner<K, V, M>, Operation<K, V>>) -> Self {
        let read = ReadHandle::new(left_right::ReadHandle::clone(&*write));

        Self { read, write }
    }

    pub fn publish(&mut self) {
        self.write.publish();
    }

    pub fn has_pending(&self) -> bool {
        self.write.has_pending_operations()
    }

    fn append_op(&mut self, op: Operation<K, V>) -> &mut Self {
        self.write.append(op);
        self
    }

    pub fn insert(&mut self, k: K, v: V) -> &mut Self {
        self.append_op(Operation::Insert(k, Aliased::from(v)))
    }

    pub fn remove(&mut self, k: K) -> &mut Self {
        self.append_op(Operation::Remove(k))
    }

    pub fn clear(&mut self) -> &mut Self {
        self.append_op(Operation::Clear)
    }
}

impl<K, V, M> Extend<(K, V)> for WriteHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

// Allow using the write handle as a read handle
impl<K, V, M> Deref for WriteHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    type Target = ReadHandle<K, V, M>;

    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
