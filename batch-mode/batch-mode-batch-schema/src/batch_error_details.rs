// ---------------- [ File: src/batch_error_details.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchErrorDetails {
    message:    String,

    #[serde(rename = "type")]
    error_type: ErrorType,
    param:      Option<String>,
    code:       Option<String>,
}

impl BatchErrorDetails {

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn error_type(&self) -> &ErrorType {
        &self.error_type
    }

    pub fn param(&self) -> Option<&str> {
        self.param.as_ref().map(|x| x.as_str())
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_ref().map(|x| x.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_details_full() {
        let json_data = json!({
            "message": "Invalid parameters",
            "type":    "parameter_error",
            "param":   "query",
            "code":    "invalid_query"
        });

        let error_details: BatchErrorDetails = serde_json::from_value(json_data).unwrap();
        assert_eq!(error_details.message(), "Invalid parameters");
        assert_eq!(error_details.error_type(), "parameter_error");
        assert_eq!(error_details.param(), Some("query"));
        assert_eq!(error_details.code(), Some("invalid_query"));
    }

    #[test]
    fn test_error_details_missing_optional_fields() {
        let json_data = json!({
            "message": "Server error",
            "type": "server_error"
            // 'param' and 'code' are missing
        });

        let error_details: BatchErrorDetails = serde_json::from_value(json_data).unwrap();
        assert_eq!(error_details.message(), "Server error");
        assert_eq!(error_details.error_type(), "server_error");
        assert_eq!(error_details.param(), None);
        assert_eq!(error_details.code(), None);
    }
}
