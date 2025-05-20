crate::ix!();

/// Error type for postal code operations.
#[derive(thiserror::Error,Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostalCodeConstructionError {
    /// The provided country is not supported by this library.
    UnsupportedCountry {
        /// Provided country that is not supported.
        attempted_country: Country,
    },
    /// The provided postal code format is invalid for the specified country.
    InvalidFormat {
        /// Provided postal code that failed validation.
        attempted_code: String,
        /// Country code attempted.
        attempted_country: Option<Country>,
    },
    /// Internal error: regex initialization failed.
    RegexInitializationError {
        /// The country whose regex failed initialization.
        country: Country,
    },
}

impl fmt::Display for PostalCodeConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostalCodeConstructionError::UnsupportedCountry { attempted_country } => {
                write!(f, "Unsupported country: {}", attempted_country)
            }
            PostalCodeConstructionError::InvalidFormat { attempted_code, attempted_country } => {
                match attempted_country {
                    Some(country) => write!(
                        f,
                        "Invalid postal code format '{}' for country '{}'",
                        attempted_code, country
                    ),
                    None => write!(
                        f,
                        "Invalid postal code format '{}', country unspecified",
                        attempted_code
                    ),
                }
            }
            PostalCodeConstructionError::RegexInitializationError { country } => {
                write!(f, "Regex initialization error for country '{}'", country)
            }
        }
    }
}

impl From<derive_builder::UninitializedFieldError> for PostalCodeConstructionError {
    fn from(_e: derive_builder::UninitializedFieldError) -> Self {
        // Convert to a suitable error, for example:
        PostalCodeConstructionError::InvalidFormat {
            attempted_code: "<unset>".to_string(),
            attempted_country: None,
        }
    }
}
