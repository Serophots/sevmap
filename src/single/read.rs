use left_right::ReadGuard;

use crate::{inner::Inner, read_ref::MapReadRef};
use std::{borrow::Borrow, hash::Hash};

/// A read handle to a single-valued map
pub struct ReadHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    handle: left_right::ReadHandle<Inner<K, V, M>>,
}

impl<K, V, M> Clone for ReadHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<K, V, M> ReadHandle<K, V, M>
where
    K: Eq + Hash + Clone,
    M: Clone,
{
    pub(crate) fn new(handle: left_right::ReadHandle<Inner<K, V, M>>) -> Self {
        ReadHandle { handle }
    }

    pub fn enter(&self) -> Option<MapReadRef<'_, K, V, M>> {
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

    fn get_raw<Q: ?Sized>(&self, key: &Q) -> Option<ReadGuard<'_, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let inner = self.handle.enter()?;
        if !inner.ready {
            return None;
        }

        ReadGuard::try_map(inner, |inner| inner.data.get(key).map(AsRef::as_ref))
    }

    #[inline]
    pub fn get<'rh, Q: ?Sized>(&'rh self, key: &'_ Q) -> Option<ReadGuard<'rh, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Call borrow here to monomorphise get_raw fewer times
        self.get_raw(key.borrow())
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.enter().map_or(false, |x| x.contains_key(key))
    }
}
