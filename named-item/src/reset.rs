crate::ix!();

/// Trait for resetting the name of an item to its default.
pub trait ResetName: SetName + DefaultName {

    /// Resets the name to the default, handling potential errors.
    fn reset_name(&mut self) -> Result<(), NameError> {
        self.set_name(&Self::default_name())
    }
}
