// ---------------- [ File: src/batch_usage.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchUsage {
    prompt_tokens:             u32,
    completion_tokens:         u32,
    total_tokens:              u32,
    prompt_tokens_details:     Option<BatchTokenDetails>,
    completion_tokens_details: Option<BatchTokenDetails>,
}

impl BatchUsage {

    pub fn prompt_tokens(&self) -> u32 {
        self.prompt_tokens
    }

    pub fn completion_tokens(&self) -> u32 {
        self.completion_tokens
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }

    // Add other accessor methods as needed
}

#[cfg(test)]
mod tests {
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
        assert_eq!(usage.prompt_tokens(), 100);
        assert_eq!(usage.completion_tokens(), 50);
        assert_eq!(usage.total_tokens(), 150);
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
