// ---------------- [ File: src/response_request_id.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResponseRequestId(String);

impl ResponseRequestId {

    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ResponseRequestId {

    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for ResponseRequestId {

    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl Display for ResponseRequestId {

    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_request_id_creation() {
        let id = ResponseRequestId::new("req_123");
        assert_eq!(id.as_str(), "req_123");
    }

    #[test]
    fn test_response_request_id_serialization() {
        let id = ResponseRequestId::new("req_456");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"req_456\"");
    }

    #[test]
    fn test_response_request_id_deserialization() {
        let json_data = "\"req_789\"";
        let id: ResponseRequestId = serde_json::from_str(json_data).unwrap();
        assert_eq!(id.as_str(), "req_789");
    }
}
