// ---------------- [ File: src/batch_token_details.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchTokenDetails {
    cached_tokens:              Option<u32>,
    audio_tokens:               Option<u32>,
    reasoning_tokens:           Option<u32>,
    accepted_prediction_tokens: Option<u32>,
    rejected_prediction_tokens: Option<u32>,
}

impl BatchTokenDetails {

    pub fn cached_tokens(&self) -> Option<u32> {
        self.cached_tokens
    }

    pub fn audio_tokens(&self) -> Option<u32> {
        self.audio_tokens
    }

    pub fn reasoning_tokens(&self) -> Option<u32> {
        self.reasoning_tokens
    }

    pub fn accepted_prediction_tokens(&self) -> Option<u32> {
        self.accepted_prediction_tokens
    }

    pub fn rejected_prediction_tokens(&self) -> Option<u32> {
        self.rejected_prediction_tokens
    }

    /// Calculates the total tokens by summing up all available token counts.
    pub fn total_tokens(&self) -> u32 {
        self.cached_tokens.unwrap_or(0)
            + self.audio_tokens.unwrap_or(0)
            + self.reasoning_tokens.unwrap_or(0)
            + self.accepted_prediction_tokens.unwrap_or(0)
            + self.rejected_prediction_tokens.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_batch_token_details_full() {
        let json_data = json!({
            "cached_tokens": 10,
            "audio_tokens": 20,
            "reasoning_tokens": 30,
            "accepted_prediction_tokens": 40,
            "rejected_prediction_tokens": 50
        });

        let token_details: BatchTokenDetails = serde_json::from_value(json_data).unwrap();

        assert_eq!(token_details.cached_tokens(), Some(10));
        assert_eq!(token_details.audio_tokens(), Some(20));
        assert_eq!(token_details.reasoning_tokens(), Some(30));
        assert_eq!(token_details.accepted_prediction_tokens(), Some(40));
        assert_eq!(token_details.rejected_prediction_tokens(), Some(50));
        assert_eq!(token_details.total_tokens(), 150);
    }

    #[test]
    fn test_batch_token_details_partial() {
        let json_data = json!({
            "audio_tokens": 25,
            "accepted_prediction_tokens": 35
            // Other fields are missing
        });

        let token_details: BatchTokenDetails = serde_json::from_value(json_data).unwrap();

        assert_eq!(token_details.cached_tokens(), None);
        assert_eq!(token_details.audio_tokens(), Some(25));
        assert_eq!(token_details.reasoning_tokens(), None);
        assert_eq!(token_details.accepted_prediction_tokens(), Some(35));
        assert_eq!(token_details.rejected_prediction_tokens(), None);
        assert_eq!(token_details.total_tokens(), 60);
    }

    #[test]
    fn test_batch_token_details_all_missing() {
        let json_data = json!({});

        let token_details: BatchTokenDetails = serde_json::from_value(json_data).unwrap();

        assert!(token_details.cached_tokens().is_none());
        assert!(token_details.audio_tokens().is_none());
        assert!(token_details.reasoning_tokens().is_none());
        assert!(token_details.accepted_prediction_tokens().is_none());
        assert!(token_details.rejected_prediction_tokens().is_none());
        assert_eq!(token_details.total_tokens(), 0);
    }
}
