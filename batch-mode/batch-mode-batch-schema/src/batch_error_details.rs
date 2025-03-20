// ---------------- [ File: src/batch_error_details.rs ]
crate::ix!();

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BatchErrorDetails {
    message:    String,

    #[serde(rename = "type")]
    error_type: ErrorType,
    param:      Option<String>,
    code:       Option<String>,
}

impl BatchErrorDetails {

    pub fn mock(custom_id: &str) -> Self {
        BatchErrorDetails {
            message:    format!("Error for {}", custom_id),
            error_type: ErrorType::Unknown("some_error".to_string()),
            param:      None,
            code:       None,
        }
    }

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
mod batch_error_details_tests {
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
        pretty_assert_eq!(error_details.message(), "Invalid parameters");
        pretty_assert_eq!(error_details.error_type(), "parameter_error");
        pretty_assert_eq!(error_details.param(), Some("query"));
        pretty_assert_eq!(error_details.code(), Some("invalid_query"));
    }

    #[test]
    fn test_error_details_missing_optional_fields() {
        let json_data = json!({
            "message": "Server error",
            "type": "server_error"
            // 'param' and 'code' are missing
        });

        let error_details: BatchErrorDetails = serde_json::from_value(json_data).unwrap();
        pretty_assert_eq!(error_details.message(), "Server error");
        pretty_assert_eq!(error_details.error_type(), "server_error");
        pretty_assert_eq!(error_details.param(), None);
        pretty_assert_eq!(error_details.code(), None);
    }
}
