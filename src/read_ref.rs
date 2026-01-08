use std::{borrow::Borrow, hash::Hash};

use left_right::ReadGuard;

use crate::inner::{Inner, Value};

pub struct MapReadRef<'rh, Key, MutV, RefV, Meta>
where
    Key: Hash + Eq,
    MutV: Clone,
    Meta: Clone,
{
    pub(crate) guard: ReadGuard<'rh, Inner<Key, MutV, RefV, Meta>>,
}

impl<'rh, Key, MutV, RefV, Meta> MapReadRef<'rh, Key, MutV, RefV, Meta>
where
    Key: Hash + Eq,
    MutV: Clone,
    Meta: Clone,
{
    // pub fn iter(&self) -> ReadGuardIter

    pub fn len(&self) -> usize {
        self.guard.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.guard.data.is_empty()
    }

    pub fn meta(&self) -> &Meta {
        &self.guard.meta
    }

    pub fn get<Q: ?Sized>(
        &'rh self,
        key: &'_ Q,
    ) -> Option<&'rh Value<MutV, RefV, crate::aliasing::NoDrop>>
    where
        Key: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.get(key)
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.contains_key(key)
    }
}
