crate::ix!();

/// Provides a history of names an item has had.
pub trait NameHistory {
    /// Adds a name to the history.
    fn add_name_to_history(&mut self, name: &str);

    /// Returns the history of names.
    fn name_history(&self) -> Vec<Cow<'_, str>>;
}

/// Trait for setting the name while maintaining a history of changes.
pub trait SetNameWithHistory: SetName + NameHistory {
    /// Sets the name and records the change in the name history.
    fn set_name_with_history(&mut self, name: &str) -> Result<(), NameError> {
        self.add_name_to_history(name);
        self.set_name(name)
    }
}
