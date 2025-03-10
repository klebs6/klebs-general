// ---------------- [ File: src/batch_success_response_body.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchSuccessResponseBody {
    id:                 String,
    object:             String,
    created:            u64,
    model:              String,
    choices:            Vec<BatchChoice>,
    usage:              BatchUsage,
    system_fingerprint: Option<String>,
}

impl BatchSuccessResponseBody {

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn object(&self) -> &str {
        &self.object
    }

    pub fn created(&self) -> u64 {
        self.created
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn choices(&self) -> &[BatchChoice] {
        &self.choices
    }

    pub fn usage(&self) -> &BatchUsage {
        &self.usage
    }

    pub fn system_fingerprint(&self) -> Option<&str> {
        self.system_fingerprint.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_body_full_deserialization() {
        let json_data = json!({
            "id": "resp_456",
            "object": "response",
            "created": 1627891234,
            "model": "test-model",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Test content",
                        "refusal": null
                    },
                    "logprobs": null,
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            },
            "system_fingerprint": "fp_abc"
        });

        let body: BatchSuccessResponseBody = serde_json::from_value(json_data).unwrap();
        assert_eq!(body.id(), "resp_456");
        assert_eq!(body.choices().len(), 1);
        assert_eq!(body.usage().total_tokens(), 150);
        assert_eq!(body.system_fingerprint(), Some("fp_abc"));
    }

    #[test]
    fn test_missing_optional_fields() {
        let json_data = json!({
            "id": "resp_789",
            "object": "response",
            "created": 1627891234,
            "model": "test-model",
            "choices": [],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
            // 'system_fingerprint' is missing
        });

        let body: BatchSuccessResponseBody = serde_json::from_value(json_data).unwrap();
        assert_eq!(body.system_fingerprint(), None);
    }

    #[test]
    fn test_missing_required_fields() {
        let json_data = json!({
            "id": "resp_000",
            "object": "response",
            "created": 1627891234,
            // 'model' is missing
            "choices": [],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        });

        let result: Result<BatchSuccessResponseBody, _> = serde_json::from_value(json_data);
        assert!(result.is_err(), "Deserialization should fail if required fields are missing");
    }
}
