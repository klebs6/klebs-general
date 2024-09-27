crate::ix!();

/// Trait for validating integrity of a component (e.g., Workspace or Crate)
pub trait ValidateIntegrity {

    type Error;

    fn validate_integrity(&self) -> Result<(), Self::Error>;
}
