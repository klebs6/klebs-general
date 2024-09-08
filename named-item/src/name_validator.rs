crate::ix!();

/// Trait for validating a name, returning a `Result`.
pub trait ValidateName {
    /// Validates a name and returns a Result indicating success or failure.
    fn validate_name(&self,name: &str) -> Result<(), NameError>;
}

/// Validator struct to validate names using a regular expression.
pub struct NameValidator {
    pattern: Regex,
}

impl NameValidator {
    /// Creates a new `NameValidator` with a regex pattern.
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
        })
    }
}

impl ValidateName for NameValidator {
    fn validate_name(&self, name: &str) -> Result<(), NameError> {
        if self.pattern.is_match(name) {
            Ok(())
        } else {
            Err(NameError::InvalidName(name.to_string()))
        }
    }
}
