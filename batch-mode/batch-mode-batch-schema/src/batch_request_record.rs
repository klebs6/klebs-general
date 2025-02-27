// ---------------- [ File: src/batch_request_record.rs ]
crate::ix!();

#[derive(Debug, Deserialize)]
pub struct BatchRequestRecord {
    id:        BatchRequestId,
    custom_id: CustomRequestId,
    prompt:    Option<String>,
    messages:  Option<Vec<String>>,
}

impl BatchRequestRecord {
    pub fn id(&self) -> &BatchRequestId {
        &self.id
    }

    pub fn custom_id(&self) -> &CustomRequestId {
        &self.custom_id
    }

    pub fn prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }

    pub fn messages(&self) -> Option<&[String]> {
        self.messages.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_batch_request_record_deserialization_full() {
        let json_data = json!({
            "id": "batch_req_123",
            "custom_id": "custom_456",
            "prompt": "Test prompt",
            "messages": ["Message 1", "Message 2"]
        });

        let record: BatchRequestRecord = serde_json::from_value(json_data).unwrap();

        assert_eq!(record.id().as_str(), "batch_req_123");
        assert_eq!(record.custom_id().as_str(), "custom_456");
        assert_eq!(record.prompt(), Some("Test prompt"));
        assert_eq!(record.messages().unwrap(), &["Message 1".to_string(), "Message 2".to_string()]);
    }

    #[test]
    fn test_batch_request_record_deserialization_partial() {
        let json_data = json!({
            "id": "batch_req_789",
            "custom_id": "custom_012"
            // 'prompt' and 'messages' are missing
        });

        let record: BatchRequestRecord = serde_json::from_value(json_data).unwrap();

        assert_eq!(record.id().as_str(), "batch_req_789");
        assert_eq!(record.custom_id().as_str(), "custom_012");
        assert!(record.prompt().is_none());
        assert!(record.messages().is_none());
    }

    #[test]
    fn test_batch_request_record_missing_required_fields() {
        let json_data = json!({
            "custom_id": "custom_345"
            // 'id' is missing
        });

        let result: Result<BatchRequestRecord, _> = serde_json::from_value(json_data);
        assert!(result.is_err(), "Deserialization should fail if required fields are missing");
    }
}
