// ---------------- [ File: src/batch_usage.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchUsage {
    prompt_tokens:             u32,
    completion_tokens:         u32,
    total_tokens:              u32,
    prompt_tokens_details:     Option<BatchTokenDetails>,
    completion_tokens_details: Option<BatchTokenDetails>,
}

impl BatchUsage {

    pub fn mock() -> Self {
        BatchUsage {
            prompt_tokens:             0,
            completion_tokens:         0,
            total_tokens:              0,
            prompt_tokens_details:     None,
            completion_tokens_details: None,
        }
    }
}

#[cfg(test)]
mod batch_usage_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_usage_deserialization() {
        let json_data = json!({
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_tokens": 150
        });

        let usage: BatchUsage = serde_json::from_value(json_data).unwrap();
        pretty_assert_eq!(usage.prompt_tokens(), 100);
        pretty_assert_eq!(usage.completion_tokens(), 50);
        pretty_assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_usage_missing_fields() {
        let json_data = json!({
            "prompt_tokens": 100,
            "total_tokens": 150
            // 'completion_tokens' is missing
        });

        let result: Result<BatchUsage, _> = serde_json::from_value(json_data);
        assert!(result.is_err(), "Deserialization should fail if fields are missing");
    }
}
