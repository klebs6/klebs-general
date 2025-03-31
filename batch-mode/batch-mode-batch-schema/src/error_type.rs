// ---------------- [ File: batch-mode-batch-schema/src/error_type.rs ]
crate::ix!();

#[derive(Debug,Clone,Eq,Hash)]
pub enum ErrorType {
    InsufficientQuota,
    InvalidRequest,
    // Add other known error types
    Unknown(String),
}

// Changed PartialEq implementation from `PartialEq<str>` to `PartialEq<&str>`
// so that comparisons like `my_error_type == "some_string"` will compile properly:
impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

// NEW: Implement PartialEq<&str> so that `ErrorType == "insufficient_quota"` works.
impl PartialEq<&str> for ErrorType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

// If you also want `ErrorType == str` (owned), you can add:
impl PartialEq<str> for ErrorType {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl ErrorType {
    pub fn as_str(&self) -> &str {
        match self {
            ErrorType::InvalidRequest => "invalid_request",
            ErrorType::InsufficientQuota => "insufficient_quota",
            ErrorType::Unknown(s) => s.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for ErrorType {
    fn deserialize<D>(deserializer: D) -> Result<ErrorType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "insufficient_quota" => Ok(ErrorType::InsufficientQuota),
            "invalid_request" => Ok(ErrorType::InvalidRequest),
            // Handle other known types
            other => Ok(ErrorType::Unknown(other.to_string())),
        }
    }
}

impl Serialize for ErrorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ErrorType::InsufficientQuota => "insufficient_quota",
            ErrorType::InvalidRequest => "invalid_request",
            ErrorType::Unknown(other) => other.as_str(),
        };
        serializer.serialize_str(s)
    }
}

#[cfg(test)]
mod error_type_tests {
    use super::*;
    
    use std::io::Write;
    
    

    #[traced_test]
    fn should_deserialize_known_error_type_insufficient_quota() {
        info!("Testing deserialization of 'insufficient_quota' into ErrorType.");
        let json_str = r#""insufficient_quota""#;
        let et: ErrorType = serde_json::from_str(json_str).expect("Failed to deserialize known error type.");
        pretty_assert_eq!(et, ErrorType::InsufficientQuota);
        debug!("Deserialized to {:?}", et);
    }

    #[traced_test]
    fn should_deserialize_known_error_type_invalid_request() {
        info!("Testing deserialization of 'invalid_request' into ErrorType.");
        let json_str = r#""invalid_request""#;
        let et: ErrorType = serde_json::from_str(json_str).expect("Failed to deserialize known error type.");
        pretty_assert_eq!(et, ErrorType::InvalidRequest);
        debug!("Deserialized to {:?}", et);
    }

    #[traced_test]
    fn should_deserialize_unknown_error_type() {
        info!("Testing deserialization of an unknown error type string.");
        let json_str = r#""some_unknown_error""#;
        let et: ErrorType = serde_json::from_str(json_str).expect("Failed to deserialize unknown error type.");
        match et {
            ErrorType::Unknown(s) => {
                pretty_assert_eq!(s, "some_unknown_error");
                trace!("Unknown variant holds the correct string: {:?}", s);
            }
            _ => panic!("Expected Unknown variant for an unrecognized string."),
        }
    }

    #[traced_test]
    fn should_serialize_insufficient_quota() {
        info!("Testing serialization of InsufficientQuota variant.");
        let et = ErrorType::InsufficientQuota;
        let serialized = serde_json::to_string(&et).expect("Failed to serialize InsufficientQuota.");
        pretty_assert_eq!(serialized, r#""insufficient_quota""#);
        debug!("Serialized to {:?}", serialized);
    }

    #[traced_test]
    fn should_serialize_invalid_request() {
        info!("Testing serialization of InvalidRequest variant.");
        let et = ErrorType::InvalidRequest;
        let serialized = serde_json::to_string(&et).expect("Failed to serialize InvalidRequest.");
        pretty_assert_eq!(serialized, r#""invalid_request""#);
        debug!("Serialized to {:?}", serialized);
    }

    #[traced_test]
    fn should_serialize_unknown() {
        info!("Testing serialization of Unknown variant.");
        let et = ErrorType::Unknown("fancy_weird_error".to_string());
        let serialized = serde_json::to_string(&et).expect("Failed to serialize Unknown variant.");
        pretty_assert_eq!(serialized, r#""fancy_weird_error""#);
        debug!("Serialized to {:?}", serialized);
    }

    #[traced_test]
    fn should_match_error_type_with_partial_eq_str() {
        info!("Testing PartialEq for ErrorType with &str.");

        let eq_insufficient = ErrorType::InsufficientQuota;
        assert!(eq_insufficient == "insufficient_quota",
            "Should match the correct string for InsufficientQuota."
        );

        let eq_invalid = ErrorType::InvalidRequest;
        assert!(eq_invalid == "invalid_request",
            "Should match the correct string for InvalidRequest."
        );

        let unknown = ErrorType::Unknown("my_unknown".to_string());
        assert!(unknown == "my_unknown",
            "Should match the correct string for Unknown variant."
        );

        warn!("PartialEq checks passed for known and unknown error types.");
    }

    #[traced_test]
    fn should_return_correct_as_str() {
        info!("Testing as_str() method on ErrorType.");

        let e_insuf = ErrorType::InsufficientQuota;
        pretty_assert_eq!(e_insuf.as_str(), "insufficient_quota");

        let e_invalid = ErrorType::InvalidRequest;
        pretty_assert_eq!(e_invalid.as_str(), "invalid_request");

        let e_unknown = ErrorType::Unknown("some_error".to_string());
        pretty_assert_eq!(e_unknown.as_str(), "some_error");
        trace!("All as_str checks passed.");
    }
}
