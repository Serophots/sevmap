pub trait Mutable<O>: Clone {
    /// Is called twice on two different allocations of self.
    ///
    /// # Safety
    /// Implementations of this function must be deterministic,
    /// otherwise state may become out of sync in the left and
    /// right maps. Take particular care with Hash, Eq traits
    /// which may not be deterministic.
    // TODO: How can we use rust to make the implementor aware that their
    // implementation of this function will be unsafe? - because it needs to
    // be deterministic.
    unsafe fn mutate(&mut self, operation: &mut O);
}

impl<T> Mutable<()> for T
where
    T: Clone,
{
    unsafe fn mutate(&mut self, _: &mut ()) {}
}
