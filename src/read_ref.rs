use std::{borrow::Borrow, collections::HashMap, hash::Hash};

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
    /// Iterate over all (keys, values) in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn iter(&self) -> ReadGuardIter<'_, Key, MutV, RefV> {
        ReadGuardIter {
            iter: self.guard.data.iter(),
        }
    }

    /// Iterate over all keys in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn keys(&self) -> ReadGuardKeys<'_, Key, MutV, RefV> {
        ReadGuardKeys {
            iter: self.guard.data.iter(),
        }
    }

    /// Iterate over all value sets in the map.
    ///
    /// Be careful with this function! While the iteration is ongoing, any writer that tries to
    /// publish changes will block waiting on this reader to finish.
    pub fn values(&self) -> ReadGuardValues<'_, Key, MutV, RefV> {
        ReadGuardValues {
            iter: self.guard.data.iter(),
        }
    }

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

/// An [`Iterator`] over (keys, values) in the map
///
/// Note: Keeps the read guard alive
pub struct ReadGuardIter<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    iter: <&'rg HashMap<Key, Value<MutV, RefV, crate::aliasing::NoDrop>> as IntoIterator>::IntoIter,
}

impl<'rg, Key, MutV, RefV> Iterator for ReadGuardIter<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    type Item = (&'rg Key, &'rg Value<MutV, RefV, crate::aliasing::NoDrop>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An [`Iterator`] over keys in the map
///
/// Note: Keeps the read guard alive
pub struct ReadGuardKeys<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    iter: <&'rg HashMap<Key, Value<MutV, RefV, crate::aliasing::NoDrop>> as IntoIterator>::IntoIter,
}

impl<'rg, Key, MutV, RefV> Iterator for ReadGuardKeys<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    type Item = &'rg Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

/// An [`Iterator`] over values in the map
///
/// Note: Keeps the read guard alive
pub struct ReadGuardValues<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    iter: <&'rg HashMap<Key, Value<MutV, RefV, crate::aliasing::NoDrop>> as IntoIterator>::IntoIter,
}

impl<'rg, Key, MutV, RefV> Iterator for ReadGuardValues<'rg, Key, MutV, RefV>
where
    MutV: Clone,
{
    type Item = &'rg Value<MutV, RefV, crate::aliasing::NoDrop>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}
