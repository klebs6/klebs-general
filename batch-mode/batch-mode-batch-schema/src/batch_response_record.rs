// ---------------- [ File: src/batch_response_record.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchResponseRecord {
    id:        BatchRequestId,
    custom_id: CustomRequestId,
    response:  BatchResponseContent,
    error:     Option<serde_json::Value>, // Assuming it's always null or can be ignored
}

impl BatchResponseRecord {

    pub fn mock_with_code_and_default_body(custom_id: &str, code: u16) -> Self {
        // (unchanged)
        let body = if code == 200 {
            json!({
                "id": "success-id",
                "object": "chat.completion",
                "created": 0,
                "model": "test-model",
                "choices": [],
                "usage": {
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "total_tokens": 0
                }
            })
        } else {
            json!({
                "error": {
                    "message": format!("Error for {custom_id}"),
                    "type": "test_error",
                    "param": null,
                    "code": null
                }
            })
        };

        BatchResponseRecord::mock_with_code_and_body(custom_id,code,&body)
    }

    pub fn mock_with_code_and_body(custom_id: &str, code: u16, body: &serde_json::Value) -> Self {
        BatchResponseRecord {
            id:        BatchRequestId::new(format!("batch_req_{custom_id}")),
            custom_id: CustomRequestId::new(custom_id),
            response:  BatchResponseContent::mock_with_code_and_body(custom_id,code,body),
            error:     None,
        }
    }

    pub fn mock(custom_id: &str) -> Self {
        BatchResponseRecord {
            id: BatchRequestId::new(format!("batch_req_{}", custom_id)),
            custom_id: CustomRequestId::new(custom_id),
            response: BatchResponseContent::mock(custom_id),
            error: None,
        }
    }

    // We revise mock_with_code(...) to produce the correct "body" automatically,
    // ensuring it includes 'id' for a success code=200 and 'error.message' for
    // code != 200. This is effectively combining the old "mock_with_code_and_body"
    // approach so that the tests won't fail on missing fields.
    //
    pub fn mock_with_code(custom_id: &str, code: u16) -> Self {

        // success:
        if code == 200 {
            let body = json!({
                "id":                 "success-id",
                "object":             "chat.completion",
                "created":            0,
                "model":              "test-model",
                "choices":            [],
                "usage": {
                    "prompt_tokens":      0,
                    "completion_tokens":  0,
                    "total_tokens":       0
                }
            });
            return BatchResponseRecord {
                id:        BatchRequestId::new(format!("batch_req_{}", custom_id)),
                custom_id: CustomRequestId::new(custom_id),
                response:  BatchResponseContent::mock_with_code_and_body(custom_id,code,&body),
                error: None,
            };
        }

        // error:
        let body = json!({
            "error": {
                "message":  format!("Error for {}", custom_id),
                "type":     "test_error",
                "param":    null,
                "code":     null
            }
        });
        BatchResponseRecord {
            id:        BatchRequestId::new(format!("batch_req_{}", custom_id)),
            custom_id: CustomRequestId::new(custom_id),
            response:  BatchResponseContent::mock_with_code_and_body(custom_id,code,&body),
            error: None,
        }
    }
}

#[cfg(test)]
mod batch_response_record_tests {
    use super::*;

    // Additional test to deserialize the provided batch line
    #[test]
    fn test_full_batch_deserialization() {
        let json = r#"
        {
            "id": "batch_req_673d5e5fc66481908be3f82f25681838",
            "custom_id": "request-0",
            "response": {
                "status_code": 200,
                "request_id": "7b003085175d218b0ceb2b79d7f60bca",
                "body": {
                    "id": "chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9",
                    "object": "chat.completion",
                    "created": 1732075005,
                    "model": "gpt-4o-2024-08-06",
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Response content here.",
                            "refusal": null
                        },
                        "logprobs": null,
                        "finish_reason": "stop"
                    }],
                    "usage": {
                        "prompt_tokens": 1528,
                        "completion_tokens": 2891,
                        "total_tokens": 4419
                    }
                }
            },
            "error": null
        }
        "#;

        let batch_response: BatchResponseRecord = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(batch_response.id, BatchRequestId::new("batch_req_673d5e5fc66481908be3f82f25681838"));
        pretty_assert_eq!(batch_response.custom_id, CustomRequestId::new("request-0"));
        assert!(batch_response.error.is_none());

        let response = batch_response.response;
        pretty_assert_eq!(response.status_code(), 200);
        pretty_assert_eq!(response.request_id(), "7b003085175d218b0ceb2b79d7f60bca");

        let body = response.body();
        pretty_assert_eq!(body.id(), Some("chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9"));
        pretty_assert_eq!(body.object(), Some("chat.completion"));
        pretty_assert_eq!(body.model(), Some("gpt-4o-2024-08-06"));

        let choices = body.choices();

        assert!(choices.is_some());
        let choices = choices.unwrap();

        pretty_assert_eq!(choices.len(), 1);

        let choice = &choices[0];
        pretty_assert_eq!(choice.index(), 0);
        pretty_assert_eq!(choice.finish_reason(), &FinishReason::Stop);
        pretty_assert_eq!(choice.message().role(), &MessageRole::Assistant);
        pretty_assert_eq!(choice.message().content(), "Response content here.");
    }
}
