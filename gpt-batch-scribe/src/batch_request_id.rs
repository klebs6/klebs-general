crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BatchRequestId(String);

impl BatchRequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BatchRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_request_id_creation() {
        let id = BatchRequestId::new("batch_123");
        assert_eq!(id.as_str(), "batch_123");
    }

    #[test]
    fn test_batch_request_id_serialization() {
        let id = BatchRequestId::new("batch_456");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"batch_456\"");
    }

    #[test]
    fn test_batch_request_id_deserialization() {
        let json_data = "\"batch_789\"";
        let id: BatchRequestId = serde_json::from_str(json_data).unwrap();
        assert_eq!(id.as_str(), "batch_789");
    }
}
