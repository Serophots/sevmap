use std::{borrow::Borrow, hash::Hash};

use left_right::ReadGuard;

use crate::inner::Inner;

pub struct MapReadRef<'rh, K, V, M>
where
    K: Hash + Eq,
    M: Clone,
{
    pub(crate) guard: ReadGuard<'rh, Inner<K, V, M>>,
}

impl<'rh, K, V, M> MapReadRef<'rh, K, V, M>
where
    K: Hash + Eq,
    M: Clone,
{
    // pub fn iter(&self) -> ReadGuardIter

    pub fn len(&self) -> usize {
        self.guard.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.guard.data.is_empty()
    }

    pub fn meta(&self) -> &M {
        &self.guard.meta
    }

    pub fn get<Q: ?Sized>(&'rh self, key: &'_ Q) -> Option<&'rh V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.get(key).map(AsRef::as_ref)
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.contains_key(key)
    }
}
