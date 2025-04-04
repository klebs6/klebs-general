// ---------------- [ File: batch-mode-batch-schema/src/batch_error_response_body.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchErrorResponseBody {
    error: BatchErrorDetails,
}

impl BatchErrorResponseBody {

    pub fn mock(custom_id: &str) -> Self {
        BatchErrorResponseBody {
            error: BatchErrorDetails::mock(custom_id),
        }
    }
}

#[cfg(test)]
mod batch_error_response_body_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_body_deserialization() {
        let json_data = json!({
            "error": {
                "message": "Invalid API key",
                "type": "authentication_error",
                "param": null,
                "code": "invalid_api_key"
            }
        });

        let body: BatchErrorResponseBody = serde_json::from_value(json_data).unwrap();
        pretty_assert_eq!(body.error().message(), "Invalid API key");
        pretty_assert_eq!(body.error().error_type(), "authentication_error");
        pretty_assert_eq!(*body.error().code(), Some("invalid_api_key".to_string()));
    }

    #[test]
    fn test_error_body_missing_fields() {
        let json_data = json!({
            "error": {
                "message": "An error occurred"
                // 'type' is missing
            }
        });

        let result: Result<BatchErrorResponseBody, _> = serde_json::from_value(json_data);
        assert!(result.is_err(), "Deserialization should fail if required fields are missing");
    }
}
