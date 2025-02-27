// ---------------- [ File: src/extract_token_name_field.rs ]
crate::ix!();

/// Extracts the "token_name" field from a JSON object.
///
/// # Arguments
/// * `json` - A reference to a `serde_json::Value` object.
///
/// # Returns
/// * `Result<&str, TokenExpanderError>` - The token name as a string slice,
///   or an error if the field is missing or invalid.
pub fn extract_token_name_field(json: &serde_json::Value) -> Result<&str, TokenParseError> {
    json.get("token_name")
        .and_then(|tn| tn.as_str())
        .ok_or(TokenParseError::InvalidTokenName)
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_token_name_valid() {
        let json = json!({ "token_name": "example_token" });
        let result = extract_token_name_field(&json);
        assert_eq!(result, Ok("example_token"));
    }

    #[test]
    fn test_extract_token_name_missing_field() {
        let json = json!({ "other_field": "value" });
        let result = extract_token_name_field(&json);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[test]
    fn test_extract_token_name_field_not_string() {
        let json = json!({ "token_name": 123 });
        let result = extract_token_name_field(&json);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[test]
    fn test_extract_token_name_empty_json() {
        let json = json!({});
        let result = extract_token_name_field(&json);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[test]
    fn test_extract_token_name_null_field() {
        let json = json!({ "token_name": null });
        let result = extract_token_name_field(&json);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }
}
