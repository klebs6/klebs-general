// ---------------- [ File: src/batch_response_body.rs ]
crate::ix!();

#[derive(Clone,Debug,Serialize)]
//#[serde(tag = "object", content = "data")]
pub enum BatchResponseBody {

    #[serde(rename = "chat.completion")]
    Success(BatchSuccessResponseBody),

    #[serde(rename = "error")]
    Error(BatchErrorResponseBody),
}

impl<'de> Deserialize<'de> for BatchResponseBody {
    fn deserialize<D>(deserializer: D) -> Result<BatchResponseBody, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        if value.get("error").is_some() {
            let error_body = BatchErrorResponseBody::deserialize(&value)
                .map_err(serde::de::Error::custom)?;
            Ok(BatchResponseBody::Error(error_body))
        } else {
            let success_body = BatchSuccessResponseBody::deserialize(&value)
                .map_err(serde::de::Error::custom)?;
            Ok(BatchResponseBody::Success(success_body))
        }
    }
}

impl BatchResponseBody {

    pub fn mock_with_code_and_body(code: u16, body: &serde_json::Value) -> Self {
        if code == 200 {
            BatchResponseBody::Success(
                serde_json::from_value(body.clone()).unwrap()
            )
        } else {
            BatchResponseBody::Error(
                serde_json::from_value(body.clone()).unwrap()
            )
        }
    }

    pub fn mock(custom_id: &str, code: u16) -> Self {
        if code == 200 {
            BatchResponseBody::Success(BatchSuccessResponseBody::mock())
        } else {
            BatchResponseBody::Error(BatchErrorResponseBody::mock(custom_id))
        }
    }

    pub fn mock_error(custom_id: &str) -> Self {
        BatchResponseBody::Error(BatchErrorResponseBody::mock(custom_id))
    }

    /// Returns `Some(&BatchSuccessResponseBody)` if the response is a success.
    pub fn as_success(&self) -> Option<&BatchSuccessResponseBody> {
        if let BatchResponseBody::Success(ref success_body) = *self {
            Some(success_body)
        } else {
            None
        }
    }

    /// Returns `Some(&BatchErrorResponseBody)` if the response is an error.
    pub fn as_error(&self) -> Option<&BatchErrorResponseBody> {
        if let BatchResponseBody::Error(ref error_body) = *self {
            Some(error_body)
        } else {
            None
        }
    }

    /// Retrieves the `id` if the response is successful.
    pub fn id(&self) -> Option<&String> {
        self.as_success().map(|body| body.id())
    }

    /// Retrieves the `object` if the response is successful.
    pub fn object(&self) -> Option<&String> {
        self.as_success().map(|body| body.object())
    }

    /// Retrieves the `model` if the response is successful.
    pub fn model(&self) -> Option<&String> {
        self.as_success().map(|body| body.model())
    }

    /// Retrieves the `choices` if the response is successful.
    pub fn choices(&self) -> Option<&Vec<BatchChoice>> {
        self.as_success().map(|body| body.choices())
    }

    /// Retrieves the `usage` if the response is successful.
    pub fn usage(&self) -> Option<&BatchUsage> {
        self.as_success().map(|body| body.usage())
    }

    /// Retrieves the `system_fingerprint` if the response is successful.
    pub fn system_fingerprint(&self) -> Option<String> {
        self.as_success().and_then(|body| body.system_fingerprint().clone())
    }
}

#[cfg(test)]
mod batch_response_body_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_body_deserialization() {

        let json_data = json!({
            "id":                 "chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9",
            "object":             "chat.completion",
            "created":            1732075005,
            "model":              "gpt-4o-2024-08-06",
            "choices":            [],
            "usage":              {
                "prompt_tokens":      40,
                "completion_tokens": 360,
                "total_tokens":      400,
            },
            "system_fingerprint": "fp_7f6be3efb0"
        });

        let body: BatchResponseBody = serde_json::from_value(json_data).unwrap();

        match body {
            BatchResponseBody::Success(success_body) => {
                pretty_assert_eq!(success_body.id(), "chatcmpl-AVW7Z2Dd49g7Zq5eVExww6dlKA8T9");
            }
            _ => panic!("Expected success body"),
        }
    }

    #[test]
    fn test_error_body_deserialization() {
        let json_data = json!({
            "error": {
                "message": "An error occurred",
                "type": "server_error",
                "param": null,
                "code": null
            }
        });

        let body: BatchResponseBody = serde_json::from_value(json_data).unwrap();
        match body {
            BatchResponseBody::Error(error_body) => {
                pretty_assert_eq!(error_body.error().message(), "An error occurred");
            }
            _ => panic!("Expected error body"),
        }
    }
}
