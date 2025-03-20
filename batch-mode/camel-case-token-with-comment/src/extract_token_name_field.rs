// ---------------- [ File: src/extract_token_name_field.rs ]
crate::ix!();

/// Extracts the "token_name" field from a JSON object.
///
/// # Arguments
/// * `json` - A reference to a `serde_json::Value` object.
///
/// # Returns
/// * `Result<&str, TokenParseError>` - The token name as a string slice,
///   or an error if the field is missing or invalid.
pub fn extract_token_name_field(json: &serde_json::Value) -> Result<&str, TokenParseError> {
    json.get("token_name")
        .and_then(|tn| tn.as_str())
        .ok_or(TokenParseError::InvalidTokenName)
}


#[cfg(test)]
mod test_extract_token_name_field {
    use super::*;

    #[traced_test]
    fn test_extract_token_name_valid() {
        tracing::info!("Testing extract_token_name_field with valid data");
        let json_data = json!({ "token_name": "example_token" });
        let result = extract_token_name_field(&json_data);
        pretty_assert_eq!(result, Ok("example_token"));
    }

    #[traced_test]
    fn test_extract_token_name_missing_field() {
        tracing::info!("Testing extract_token_name_field with missing field");
        let json_data = json!({ "other_field": "value" });
        let result = extract_token_name_field(&json_data);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[traced_test]
    fn test_extract_token_name_field_not_string() {
        tracing::info!("Testing extract_token_name_field with non-string field");
        let json_data = json!({ "token_name": 123 });
        let result = extract_token_name_field(&json_data);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[traced_test]
    fn test_extract_token_name_empty_json() {
        tracing::info!("Testing extract_token_name_field with empty JSON object");
        let json_data = json!({});
        let result = extract_token_name_field(&json_data);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }

    #[traced_test]
    fn test_extract_token_name_null_field() {
        tracing::info!("Testing extract_token_name_field with null token_name field");
        let json_data = json!({ "token_name": null });
        let result = extract_token_name_field(&json_data);
        assert!(matches!(result, Err(TokenParseError::InvalidTokenName)));
    }
}
