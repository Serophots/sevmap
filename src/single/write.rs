use left_right::aliasing::Aliased;

use crate::{
    get_mut::Mutable,
    inner::{Inner, Operation, Value},
    single::read::ReadHandle,
};
use std::{hash::Hash, ops::Deref};

/// A write handle to a single-valued map
pub struct WriteHandle<Key, MutV, RefV, Meta, Op>
where
    Key: Eq + Hash + Clone,
    MutV: Mutable<Op> + Clone,
    Meta: Clone,
{
    write: left_right::WriteHandle<Inner<Key, MutV, RefV, Meta>, Operation<Key, MutV, RefV, Op>>,
    read: ReadHandle<Key, MutV, RefV, Meta>,
}

impl<Key, MutV, RefV, Meta, Op> WriteHandle<Key, MutV, RefV, Meta, Op>
where
    Key: Eq + Hash + Clone,
    MutV: Mutable<Op> + Clone,
    Meta: Clone,
{
    pub(crate) fn new(
        write: left_right::WriteHandle<
            Inner<Key, MutV, RefV, Meta>,
            Operation<Key, MutV, RefV, Op>,
        >,
    ) -> Self {
        let read = ReadHandle::new(left_right::ReadHandle::clone(&*write));

        Self { read, write }
    }

    pub fn publish(&mut self) {
        self.write.publish();
    }

    pub fn has_pending(&self) -> bool {
        self.write.has_pending_operations()
    }

    fn append_op(&mut self, op: Operation<Key, MutV, RefV, Op>) -> &mut Self {
        self.write.append(op);
        self
    }

    pub fn insert(&mut self, k: Key, ref_v: RefV, mut_v: MutV) -> &mut Self {
        let value = Value {
            mut_v,
            ref_v: Aliased::from(ref_v),
        };

        self.append_op(Operation::Insert(k, value))
    }

    pub fn remove(&mut self, k: Key) -> &mut Self {
        self.append_op(Operation::Remove(k))
    }

    pub fn clear(&mut self) -> &mut Self {
        self.append_op(Operation::Clear)
    }
}

impl<Key, MutV, RefV, Meta, Op> Extend<(Key, (RefV, MutV))>
    for WriteHandle<Key, MutV, RefV, Meta, Op>
where
    Key: Eq + Hash + Clone,
    MutV: Mutable<Op> + Clone,
    Meta: Clone,
{
    fn extend<T: IntoIterator<Item = (Key, (RefV, MutV))>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v.0, v.1);
        }
    }
}

// Allow using the write handle as a read handle
impl<Key, MutV, RefV, Meta, Op> Deref for WriteHandle<Key, MutV, RefV, Meta, Op>
where
    Key: Eq + Hash + Clone,
    MutV: Mutable<Op> + Clone,
    Meta: Clone,
{
    type Target = ReadHandle<Key, MutV, RefV, Meta>;

    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
