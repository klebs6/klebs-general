// ---------------- [ File: src/finish_reason.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Unknown(String),
}

impl<'de> Deserialize<'de> for FinishReason {
    fn deserialize<D>(deserializer: D) -> Result<FinishReason, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt.as_deref() {
            Some("stop")           => Ok(FinishReason::Stop),
            Some("length")         => Ok(FinishReason::Length),
            Some("content_filter") => Ok(FinishReason::ContentFilter),
            Some(other)            => Ok(FinishReason::Unknown(other.to_string())),
            None                   => Ok(FinishReason::Unknown("None".to_string())),
        }
    }
}

impl Serialize for FinishReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            FinishReason::Stop => "stop",
            FinishReason::Length => "length",
            FinishReason::ContentFilter => "content_filter",
            FinishReason::Unknown(other) => other.as_str(),
        };
        serializer.serialize_some(s)
    }
}

#[cfg(test)]
mod finish_reason_tests {
    use super::*;

    // Test suite for FinishReason
    #[test]
    fn test_finish_reason_deserialization() {
        // Known reasons
        let reasons = vec!["stop", "length", "content_filter"];
        let expected_reasons = vec![
            FinishReason::Stop,
            FinishReason::Length,
            FinishReason::ContentFilter,
        ];

        for (reason_str, expected_reason) in reasons.iter().zip(expected_reasons.iter()) {
            let json = format!("\"{}\"", reason_str);
            let finish_reason: FinishReason = serde_json::from_str(&json).unwrap();
            pretty_assert_eq!(&finish_reason, expected_reason);
        }

        // Unknown reason
        let json = "\"unknown_reason\"";
        let finish_reason: FinishReason = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(
            finish_reason,
            FinishReason::Unknown("unknown_reason".to_string())
        );

        // Null reason
        let json = "null";
        let finish_reason: FinishReason = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(
            finish_reason,
            FinishReason::Unknown("None".to_string())
        );

        // Empty string as reason
        let json = "\"\"";
        let finish_reason: FinishReason = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(finish_reason, FinishReason::Unknown("".to_string()));
    }
}
