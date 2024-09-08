crate::ix!();

/// Trait for supporting names in multiple languages.
pub trait MultilingualName {
    /// Sets the name in a specific language.
    fn set_name_in_language(&mut self, language: &str, name: &str);

    /// Gets the name in a specific language.
    fn name_in_language(&self, language: &str) -> Option<Cow<'_, str>>;
}
