// ---------------- [ File: src/batch_response_record.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchResponseRecord {
    id:        BatchRequestId,
    custom_id: CustomRequestId,
    response:  BatchResponseContent,
    error:     Option<serde_json::Value>, // Assuming it's always null or can be ignored
}

impl BatchResponseRecord {

    pub fn id(&self) -> &BatchRequestId {
        &self.id
    }

    pub fn custom_id(&self) -> &CustomRequestId {
        &self.custom_id
    }

    pub fn response(&self) -> &BatchResponseContent {
        &self.response
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(batch_response.id, BatchRequestId::new("batch_req_673d5e5fc66481908be3f82f25681838"));
        assert_eq!(batch_response.custom_id, CustomRequestId::new("request-0"));
        assert!(batch_response.error.is_none());

        let response = batch_response.response;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.request_id(), "7b003085175d218b0ceb2b79d7f60bca");

        let body = response.body();
        assert_eq!(body.id(), Some("chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9"));
        assert_eq!(body.object(), Some("chat.completion"));
        assert_eq!(body.model(), Some("gpt-4o-2024-08-06"));

        let choices = body.choices();

        assert!(choices.is_some());
        let choices = choices.unwrap();

        assert_eq!(choices.len(), 1);

        let choice = &choices[0];
        assert_eq!(choice.index(), 0);
        assert_eq!(choice.finish_reason(), &FinishReason::Stop);
        assert_eq!(choice.message().role(), &MessageRole::Assistant);
        assert_eq!(choice.message().content(), "Response content here.");
    }
}
