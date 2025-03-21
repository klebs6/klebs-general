// ---------------- [ File: src/batch_request_record.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchRequestRecord {
    id:        BatchRequestId,
    custom_id: CustomRequestId,
    prompt:    Option<String>,
    messages:  Option<Vec<String>>,
}

#[cfg(test)]
mod batch_request_record_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    fn test_batch_request_record_deserialization_full() {
        info!("Starting test: test_batch_request_record_deserialization_full");

        let json_data = json!({
            "id": "batch_req_123",
            "custom_id": "custom_456",
            "prompt": "Test prompt",
            "messages": ["Message 1", "Message 2"]
        });

        let record: BatchRequestRecord = serde_json::from_value(json_data).unwrap();

        pretty_assert_eq!(record.id().as_str(), "batch_req_123");
        pretty_assert_eq!(record.custom_id().as_str(), "custom_456");
        pretty_assert_eq!(*record.prompt(), Some("Test prompt".to_string()));

        // FIX: do not unwrap() by value. We do as_ref().unwrap() to avoid moving
        pretty_assert_eq!(
            record.messages().as_ref().unwrap(),
            &["Message 1".to_string(), "Message 2".to_string()]
        );

        info!("Finished test: test_batch_request_record_deserialization_full");
    }

    #[test]
    fn test_batch_request_record_deserialization_partial() {
        let json_data = json!({
            "id": "batch_req_789",
            "custom_id": "custom_012"
            // 'prompt' and 'messages' are missing
        });

        let record: BatchRequestRecord = serde_json::from_value(json_data).unwrap();

        pretty_assert_eq!(record.id().as_str(), "batch_req_789");
        pretty_assert_eq!(record.custom_id().as_str(), "custom_012");
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
