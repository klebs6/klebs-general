// ---------------- [ File: src/batch_error_details.rs ]
crate::ix!();

#[derive(Getters,Builder,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchErrorDetails {
    message:    String,

    #[serde(rename = "type")]
    error_type: ErrorType,

    #[builder(default)]
    param:      Option<String>,

    #[builder(default)]
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
        pretty_assert_eq!(*error_details.param(), Some("query".to_string()));
        pretty_assert_eq!(*error_details.code(), Some("invalid_query".to_string()));
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
        pretty_assert_eq!(*error_details.param(), None);
        pretty_assert_eq!(*error_details.code(), None);
    }
}
