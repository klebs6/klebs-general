crate::ix!();

/// Trait for namespacing names.
pub trait NamespaceName: Named {
    /// Returns the namespace of the name.
    fn namespace(&self) -> Cow<'_, str>;

    /// Returns the fully qualified name, including the namespace.
    fn full_name(&self) -> Cow<'_, str> {
        Cow::from(format!("{}::{}", self.namespace(), self.name()))
    }
}
