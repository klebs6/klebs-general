// ---------------- [ File: src/batch_response_content.rs ]
crate::ix!();

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BatchResponseContent {
    status_code: u16,
    request_id:  ResponseRequestId,
    body:        BatchResponseBody,
}

impl BatchResponseContent {

    pub fn mock_with_code_and_body(custom_id: &str, code: u16, body: &serde_json::Value) -> Self {
        BatchResponseContent {
            status_code: code,
            request_id:  ResponseRequestId::new(format!("resp_req_{custom_id}")),
            body:        BatchResponseBody::mock_with_code_and_body(code,body),
        }
    }

    pub fn mock_with_code(custom_id: &str, code: u16) -> Self {
        BatchResponseContent {
            status_code: code,
            request_id:  ResponseRequestId::new(format!("req_resp_{}", custom_id)),
            body:        BatchResponseBody::mock(custom_id, code),
        }
    }

    pub fn mock(custom_id: &str) -> Self {
        BatchResponseContent {
            status_code: 400,
            request_id: ResponseRequestId::new(format!("resp_req_{}", custom_id)),
            body:       BatchResponseBody::mock_error(custom_id),
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn request_id(&self) -> &ResponseRequestId {
        &self.request_id
    }

    pub fn body(&self) -> &BatchResponseBody {
        &self.body
    }

    /// Checks if the response indicates success (status code 200)
    pub fn is_success(&self) -> bool {
        self.status_code == 200
    }

    /// Retrieves the success body if present
    pub fn success_body(&self) -> Option<&BatchSuccessResponseBody> {
        match &self.body {
            BatchResponseBody::Success(body) => Some(body),
            _ => None,
        }
    }

    /// Retrieves the error body if present
    pub fn error_body(&self) -> Option<&BatchErrorResponseBody> {
        match &self.body {
            BatchResponseBody::Error(body) => Some(body),
            _ => None,
        }
    }
}

#[cfg(test)]
mod batch_response_content_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response_content() {
        let json_data = json!({
            "status_code": 200,
            "request_id": "req_123",
            "body": {
                "id": "resp_456",
                "object": "response",
                "created": 1627891234,
                "model": "test-model",
                "choices": [],
                "usage": {
                    "prompt_tokens": 100,
                    "completion_tokens": 50,
                    "total_tokens": 150
                },
                "system_fingerprint": "fp_abc"
            }
        });

        let response: BatchResponseContent = serde_json::from_value(json_data).unwrap();
        pretty_assert_eq!(response.status_code(), 200);
        assert!(response.is_success());
        pretty_assert_eq!(response.request_id().as_str(), "req_123");
        assert!(response.success_body().is_some());
        assert!(response.error_body().is_none());
    }

    #[test]
    fn test_error_response_content() {
        let json_data = json!({
            "status_code": 400,
            "request_id": "req_789",
            "body": {
                "error": {
                    "message": "Invalid request",
                    "type": "invalid_request_error",
                    "param": "prompt",
                    "code": "invalid_prompt"
                }
            }
        });

        let response: BatchResponseContent = serde_json::from_value(json_data).unwrap();
        pretty_assert_eq!(response.status_code(), 400);
        assert!(!response.is_success());
        pretty_assert_eq!(response.request_id().as_str(), "req_789");
        assert!(response.success_body().is_none());
        assert!(response.error_body().is_some());
    }

    #[test]
    fn test_missing_body() {
        let json_data = json!({
            "status_code": 500,
            "request_id": "req_000",
            "body": {}
        });

        let result: Result<BatchResponseContent, _> = serde_json::from_value(json_data);
        assert!(result.is_err(), "Deserialization should fail if body is invalid");
    }

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
                    },
                    "system_fingerprint": "fp_7f6be3efb0"
                }
            },
            "error": null
        }
        "#;

        // Assuming you have defined BatchResponseRecord and related structs
        let batch_response: BatchResponseRecord = serde_json::from_str(json).unwrap();
        let response = batch_response.response();

        // Accessing fields directly through BatchResponseBody methods
        let body = response.body();

        pretty_assert_eq!(body.id(), Some("chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9"));
        pretty_assert_eq!(body.object(), Some("chat.completion"));
        pretty_assert_eq!(body.model(), Some("gpt-4o-2024-08-06"));
        pretty_assert_eq!(body.system_fingerprint(), Some("fp_7f6be3efb0"));

        // Access choices
        if let Some(choices) = body.choices() {
            pretty_assert_eq!(choices.len(), 1);
            let choice = &choices[0];
            pretty_assert_eq!(choice.index(), 0);
            pretty_assert_eq!(choice.finish_reason(), &FinishReason::Stop);
            pretty_assert_eq!(choice.message().content(), "Response content here.");
        } else {
            panic!("Expected choices in the response body");
        }
    }
}
