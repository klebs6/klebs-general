crate::ix!();

/// Trait for normalizing the case of a name (e.g., lowercase).
pub trait NormalizeName {
    /// Normalizes the name to a consistent case (e.g., lowercase).
    fn normalize_name(&mut self) -> Result<(), NameError>;
}


