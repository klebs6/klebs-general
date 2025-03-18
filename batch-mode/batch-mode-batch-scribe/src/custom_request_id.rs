// ---------------- [ File: src/custom_request_id.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomRequestId(String);

impl CustomRequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CustomRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod custom_request_id_tests {
    use super::*;

    #[test]
    fn test_custom_request_id_creation() {
        let id = CustomRequestId::new("custom_123");
        assert_eq!(id.as_str(), "custom_123");
    }

    #[test]
    fn test_custom_request_id_serialization() {
        let id = CustomRequestId::new("custom_456");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"custom_456\"");
    }

    #[test]
    fn test_custom_request_id_deserialization() {
        let json_data = "\"custom_789\"";
        let id: CustomRequestId = serde_json::from_str(json_data).unwrap();
        assert_eq!(id.as_str(), "custom_789");
    }
}
