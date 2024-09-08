crate::ix!();

/// Trait for handling multiple names or aliases.
pub trait NamedAlias {
    /// Adds an alias for the item.
    fn add_alias(&mut self, alias: &str);

    /// Returns the list of aliases.
    fn aliases(&self) -> Vec<Cow<'_, str>>;
}
