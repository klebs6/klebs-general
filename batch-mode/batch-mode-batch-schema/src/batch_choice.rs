// ---------------- [ File: batch-mode-batch-schema/src/batch_choice.rs ]
crate::ix!();

#[derive(Getters,Builder,Clone,Debug,Serialize)]
#[builder(setter(into))]
#[getset(get="pub")]
#[serde(deny_unknown_fields)]
pub struct BatchChoice {
    index:         u32,
    message:       BatchMessage,
    logprobs:      Option<serde_json::Value>,
    finish_reason: FinishReason,
}

impl<'de> Deserialize<'de> for BatchChoice {

    fn deserialize<D>(deserializer: D) -> Result<BatchChoice, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BatchChoiceHelper {
            index:         u32,
            message:       BatchMessage,
            logprobs:      Option<serde_json::Value>,
            finish_reason: FinishReason,
        }

        let helper = BatchChoiceHelper::deserialize(deserializer)?;

        // Validate `logprobs`
        if let Some(ref logprobs) = helper.logprobs {
            if !logprobs.is_object() && !logprobs.is_null() {
                return Err(de::Error::custom("`logprobs` must be an object or null"));
            }
        }

        Ok(BatchChoice {
            index: helper.index,
            message: helper.message,
            logprobs: helper.logprobs,
            finish_reason: helper.finish_reason,
        })
    }
}

#[cfg(test)]
mod batch_choice_tests {
    use super::*;

    #[test]
    fn test_invalid_index_type() {
        let json = r#"{
            "index": "invalid_index",
            "message": {
                "role": "assistant",
                "content": "Invalid index type.",
                "refusal": null
            },
            "logprobs": null,
            "finish_reason": "stop"
        }"#;

        let result: Result<BatchChoice, _> = serde_json::from_str(json);
        println!("Result: {:?}", result);
        assert!(result.is_err());
    }

    // Test suite for BatchChoice
    #[test]
    fn test_batch_choice_deserialization() {
        // Choice with all fields present
        let json = r#"{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "This is the assistant's response.",
                "refusal": null
            },
            "logprobs": null,
            "finish_reason": "stop"
        }"#;
        let choice: BatchChoice = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(*choice.index(), 0);
        pretty_assert_eq!(choice.message().role(), &MessageRole::Assistant);
        pretty_assert_eq!(choice.finish_reason(), &FinishReason::Stop);
        assert!(choice.logprobs.is_none());

        // Choice with logprobs present
        let json = r#"{
            "index": 1,
            "message": {
                "role": "assistant",
                "content": "Another response.",
                "refusal": null
            },
            "logprobs": {
                "tokens": ["This", "is", "a", "test"],
                "token_logprobs": [-0.1, -0.2, -0.3, -0.4]
            },
            "finish_reason": "length"
        }"#;
        let choice: BatchChoice = serde_json::from_str(json).unwrap();
        assert!(choice.logprobs.is_some());
        pretty_assert_eq!(choice.finish_reason(), &FinishReason::Length);

        // Choice with unknown finish_reason
        let json = r#"{
            "index": 2,
            "message": {
                "role": "assistant",
                "content": "Response with unknown finish reason.",
                "refusal": null
            },
            "logprobs": null,
            "finish_reason": "unknown_reason"
        }"#;
        let choice: BatchChoice = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(
            choice.finish_reason(),
            &FinishReason::Unknown("unknown_reason".to_string())
        );

        // Choice with missing finish_reason field
        let json = r#"{
            "index": 3,
            "message": {
                "role": "assistant",
                "content": "Response without finish_reason.",
                "refusal": null
            },
            "logprobs": null
        }"#;
        let choice: BatchChoice = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(
            choice.finish_reason(),
            &FinishReason::Unknown("None".to_string())
        );

        // Choice with missing optional fields
        let json = r#"{
            "index": 4,
            "message": {
                "role": "assistant",
                "content": "Response with minimal fields."
            }
        }"#;
        let choice: BatchChoice = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(*choice.index(), 4);
        pretty_assert_eq!(choice.message().content(), "Response with minimal fields.");
        assert!(choice.logprobs.is_none());
        pretty_assert_eq!(
            choice.finish_reason(),
            &FinishReason::Unknown("None".to_string())
        );

        // Choice with invalid index type
        let json = r#"{
            "index": "invalid_index",
            "message": {
                "role": "assistant",
                "content": "Invalid index type.",
                "refusal": null
            },
            "logprobs": null,
            "finish_reason": "stop"
        }"#;
        let result: Result<BatchChoice, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Choice with missing message field
        let json = r#"{
            "index": 5,
            "logprobs": null,
            "finish_reason": "stop"
        }"#;
        let result: Result<BatchChoice, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Choice with invalid logprobs type
        let json = r#"{
            "index": 6,
            "message": {
                "role": "assistant",
                "content": "Invalid logprobs.",
                "refusal": null
            },
            "logprobs": "invalid_logprobs",
            "finish_reason": "stop"
        }"#;
        let result: Result<BatchChoice, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
