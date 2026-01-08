pub trait Mutable<O>: Clone {
    /// Similar to left_right's Absorb trait. See that for
    /// details.
    ///
    /// # Safety
    /// Implementations of this function must be deterministic,
    /// otherwise state may become out of sync in the left and
    /// right maps. Take particular care with Hash, Eq traits
    /// which may not be deterministic.
    fn mutate_first(&mut self, operation: &mut O);

    /// Similar to left_right's Absorb trait. See that for
    /// details.
    ///
    /// # Safety
    /// Implementations of this function must be deterministic,
    /// otherwise state may become out of sync in the left and
    /// right maps. Take particular care with Hash, Eq traits
    /// which may not be deterministic.
    fn mutate_second(&mut self, mut operation: O) {
        Mutable::mutate_first(self, &mut operation);
    }
}

impl<T> Mutable<()> for T
where
    T: Clone,
{
    fn mutate_first(&mut self, _: &mut ()) {}

    fn mutate_second(&mut self, _: ()) {}
}
