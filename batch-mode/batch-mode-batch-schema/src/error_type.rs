// ---------------- [ File: src/error_type.rs ]
crate::ix!();

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum ErrorType {
    InsufficientQuota,
    InvalidRequest,
    // Add other known error types
    Unknown(String),
}

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
