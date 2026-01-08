use left_right::ReadGuard;

use crate::{
    inner::{Inner, Value},
    read_ref::MapReadRef,
};
use std::{borrow::Borrow, hash::Hash};

/// A read handle to a single-valued map
pub struct ReadHandle<Key, MutV, RefV, Meta>
where
    Key: Eq + Hash + Clone,
    MutV: Clone,
    Meta: Clone,
{
    handle: left_right::ReadHandle<Inner<Key, MutV, RefV, Meta>>,
}

impl<Key, MutV, RefV, Meta> Clone for ReadHandle<Key, MutV, RefV, Meta>
where
    Key: Eq + Hash + Clone,
    MutV: Clone,
    Meta: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<Key, MutV, RefV, Meta> ReadHandle<Key, MutV, RefV, Meta>
where
    Key: Eq + Hash + Clone,
    MutV: Clone,
    Meta: Clone,
{
    pub(crate) fn new(handle: left_right::ReadHandle<Inner<Key, MutV, RefV, Meta>>) -> Self {
        ReadHandle { handle }
    }

    pub fn enter(&self) -> Option<MapReadRef<'_, Key, MutV, RefV, Meta>> {
        let guard = self.handle.enter()?;
        if !guard.ready {
            return None;
        }

        Some(MapReadRef { guard })
    }

    pub fn len(&self) -> usize {
        self.enter().map_or(0, |x| x.len())
    }

    pub fn is_empty(&self) -> bool {
        self.enter().map_or(false, |x| x.is_empty())
    }

    fn get_raw<Q: ?Sized>(
        &self,
        key: &Q,
    ) -> Option<ReadGuard<'_, Value<MutV, RefV, crate::aliasing::NoDrop>>>
    where
        Key: Borrow<Q>,
        Q: Hash + Eq,
    {
        let inner = self.handle.enter()?;
        if !inner.ready {
            return None;
        }

        ReadGuard::try_map(inner, |inner| inner.data.get(key))
    }

    #[inline]
    pub fn get<'rh, Q: ?Sized>(
        &'rh self,
        key: &'_ Q,
    ) -> Option<ReadGuard<'rh, Value<MutV, RefV, crate::aliasing::NoDrop>>>
    where
        Key: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Call borrow here to monomorphise get_raw fewer times
        self.get_raw(key.borrow())
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.enter().map_or(false, |x| x.contains_key(key))
    }
}
