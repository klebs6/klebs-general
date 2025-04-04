// ---------------- [ File: batch-mode-batch-schema/src/batch_success_response_body.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchSuccessResponseBody {
    id:      String,
    object:  String,
    created: u64,
    model:   String,
    choices: Vec<BatchChoice>,
    usage:   BatchUsage,

    #[builder(default)]
    system_fingerprint: Option<String>,
}

impl Default for BatchSuccessResponseBody {
    fn default() -> Self {
        Self {
            id:                 "generated-id".into(),
            object:             "chat.completion".into(),
            created:            0,
            model:              "mock-model".into(),
            choices:            vec![],
            usage:              BatchUsage::mock(),
            system_fingerprint: None,
        }
    }
}

impl BatchSuccessResponseBody {

    pub fn mock() -> Self {
        info!("Using updated mock to produce recognized 'chat.completion' object for success scenario.");

        BatchSuccessResponseBody {
            id:                 "success-id".to_string(),
            object:             "chat.completion".to_string(),
            created:            0,
            model:              "test-model".to_string(),
            choices:            vec![],
            usage:              BatchUsage::mock(),
            system_fingerprint: None,
        }
    }
}

#[cfg(test)]
mod batch_success_response_body_tests {
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
        pretty_assert_eq!(body.id(), "resp_456");
        pretty_assert_eq!(body.choices().len(), 1);
        pretty_assert_eq!(*body.usage().total_tokens(), 150);
        pretty_assert_eq!(*body.system_fingerprint(), Some("fp_abc".to_string()));
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
        pretty_assert_eq!(*body.system_fingerprint(), None);
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
