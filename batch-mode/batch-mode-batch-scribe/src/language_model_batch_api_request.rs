// ---------------- [ File: src/language_model_batch_api_request.rs ]
/*!
  | basic example:
  | {
  |     "custom_id": "request-1", 
  |     "method": "POST", 
  |     "url": "/v1/chat/completions", 
  |     "body": {
  |         "model": "gpt-4", 
  |         "messages": [
  |             {"role": "system", "content": "You are a helpful assistant."},
  |             {"role": "user", "content": "Hello world!"}
  |         ],
  |         "max_tokens": 1000
  |     }
  | }
  */
crate::ix!();

/// Represents the complete request structure.
#[derive(Getters,Setters,Clone,Debug, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct LanguageModelBatchAPIRequest {

    /// Identifier for the custom request.
    custom_id: CustomRequestId,

    /// HTTP method used for the request.
    #[serde(with = "http_method")]
    method: HttpMethod,

    /// URL of the API endpoint.
    #[serde(with = "api_url")]
    url:  LanguageModelApiUrl,

    /// Body of the request.
    body: LanguageModelRequestBody,
}

impl LanguageModelBatchAPIRequest {

    pub fn mock(custom_id: &str) -> Self {
        LanguageModelBatchAPIRequest {
            custom_id: CustomRequestId::new(custom_id),
            method:    HttpMethod::Post,
            url:       LanguageModelApiUrl::ChatCompletions,
            body:      LanguageModelRequestBody::mock(),
        }
    }
}

impl From<LanguageModelBatchAPIRequest> for BatchRequestInput {

    fn from(request: LanguageModelBatchAPIRequest) -> Self {
        BatchRequestInput {
            custom_id: request.custom_id.to_string(),
            method: BatchRequestInputMethod::POST,
            url: match request.url {
                LanguageModelApiUrl::ChatCompletions => BatchEndpoint::V1ChatCompletions,
            },
            body: Some(serde_json::to_value(&request.body).unwrap()),
        }
    }
}

pub fn create_batch_input_file(
    requests:             &[LanguageModelBatchAPIRequest],
    batch_input_filename: impl AsRef<Path>,

) -> Result<(), BatchInputCreationError> {

    use std::io::{BufWriter,Write};
    use std::fs::File;

    let file = File::create(batch_input_filename.as_ref())?;
    let mut writer = BufWriter::new(file);

    for request in requests {
        let batch_input = BatchRequestInput {
            custom_id: request.custom_id.to_string(),
            method: match request.method {
                HttpMethod::Post => BatchRequestInputMethod::POST,
                _ => unimplemented!("Only POST method is supported"),
            },
            url: match request.url {
                LanguageModelApiUrl::ChatCompletions => BatchEndpoint::V1ChatCompletions,
                // Handle other endpoints if necessary
            },
            body: Some(serde_json::to_value(&request.body)?),
        };
        let line = serde_json::to_string(&batch_input)?;
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

impl LanguageModelBatchAPIRequest {

    pub fn requests_from_query_strings(system_message: &str, model: LanguageModelType, queries: &[String]) -> Vec<Self> {
        queries.iter().enumerate().map(|(idx,query)| Self::new_basic(model,idx,system_message,&query)).collect()
    }

    pub fn new_basic(model: LanguageModelType, idx: usize, system_message: &str, user_message: &str) -> Self {
        Self {
            custom_id: Self::custom_id_for_idx(idx),
            method:    HttpMethod::Post,
            url:       LanguageModelApiUrl::ChatCompletions,
            body:      LanguageModelRequestBody::new_basic(model,system_message,user_message),
        }
    }

    pub fn new_with_image(model: LanguageModelType, idx: usize, system_message: &str, user_message: &str, image_b64: &str) -> Self {
        Self {
            custom_id: Self::custom_id_for_idx(idx),
            method:    HttpMethod::Post,
            url:       LanguageModelApiUrl::ChatCompletions,
            body:      LanguageModelRequestBody::new_with_image(model,system_message,user_message,image_b64),
        }
    }
}

impl Display for LanguageModelBatchAPIRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => {
                // Handle JSON serialization errors, though they shouldn't occur with proper struct definitions
                write!(f, "Error serializing to JSON: {}", e)
            }
        }
    }
}

impl LanguageModelBatchAPIRequest {

    pub(crate) fn custom_id_for_idx(idx: usize) -> CustomRequestId {
        CustomRequestId::new(format!("request-{}",idx))
    }
}

/// Custom serialization modules for enum string representations.
mod http_method {

    use super::*;

    pub fn serialize<S>(value: &HttpMethod, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HttpMethod, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "POST" => Ok(HttpMethod::Post),
            _ => Err(serde::de::Error::custom("unknown method")),
        }
    }
}

mod api_url {

    use super::*;

    pub fn serialize<S>(value: &LanguageModelApiUrl, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<LanguageModelApiUrl, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "/v1/chat/completions" => Ok(LanguageModelApiUrl::ChatCompletions),
            _ => Err(serde::de::Error::custom("unknown URL")),
        }
    }
}

// Updated: Provide a minimal request body that matches the struct shape.
pub fn make_valid_lmb_api_request_json_mock(custom_id: &str) -> String {
    let request = LanguageModelBatchAPIRequest::mock(custom_id);
    serde_json::to_string(&request).unwrap()
}

#[cfg(test)]
mod language_model_batch_api_request_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn mock_produces_expected_fields() {
        trace!("===== BEGIN TEST: mock_produces_expected_fields =====");
        let custom_id_str = "test_id";
        let request = LanguageModelBatchAPIRequest::mock(custom_id_str);
        debug!("Mock request: {:?}", request);

        pretty_assert_eq!(request.custom_id().to_string(), custom_id_str, "Custom ID mismatch");
        match request.method {
            HttpMethod::Post => trace!("Method is POST as expected"),
            _ => panic!("Expected POST method"),
        }
        match request.url {
            LanguageModelApiUrl::ChatCompletions => trace!("URL is ChatCompletions as expected"),
        }
        let body = &request.body;
        match body.model() {
            LanguageModelType::Gpt4o => trace!("Body model is Gpt4o as expected"),
            _ => panic!("Expected LanguageModelType::Gpt4o"),
        }
        assert!(body.messages().is_empty(), "Mock body should start with no messages");
        pretty_assert_eq!(
            *body.max_completion_tokens(), 128,
            "Mock body should have max_completion_tokens=128"
        );

        trace!("===== END TEST: mock_produces_expected_fields =====");
    }

    #[traced_test]
    fn custom_id_for_idx_produces_expected_format() {
        trace!("===== BEGIN TEST: custom_id_for_idx_produces_expected_format =====");
        let idx = 5;
        let custom_id = LanguageModelBatchAPIRequest::custom_id_for_idx(idx);
        debug!("Produced CustomRequestId: {:?}", custom_id);
        pretty_assert_eq!(
            custom_id.to_string(),
            "request-5",
            "Expected custom ID format 'request-<idx>'"
        );
        trace!("===== END TEST: custom_id_for_idx_produces_expected_format =====");
    }

    #[traced_test]
    fn new_basic_produces_correct_fields() {
        trace!("===== BEGIN TEST: new_basic_produces_correct_fields =====");
        let idx = 2;
        let model = LanguageModelType::Gpt4o;
        let system_msg = "System basic";
        let user_msg = "User basic request";
        let request = LanguageModelBatchAPIRequest::new_basic(model.clone(), idx, system_msg, user_msg);
        debug!("Constructed request: {:?}", request);

        pretty_assert_eq!(request.custom_id().to_string(), "request-2");
        match request.method {
            HttpMethod::Post => trace!("Method is POST as expected"),
            _ => panic!("Expected POST method"),
        }
        match request.url {
            LanguageModelApiUrl::ChatCompletions => trace!("URL is ChatCompletions as expected"),
        }
        pretty_assert_eq!(
            request.body.messages().len(),
            2,
            "Should have system + user messages"
        );

        trace!("===== END TEST: new_basic_produces_correct_fields =====");
    }

    #[traced_test]
    fn new_with_image_produces_correct_fields() {
        trace!("===== BEGIN TEST: new_with_image_produces_correct_fields =====");
        let idx = 3;
        let model = LanguageModelType::Gpt4o;
        let system_msg = "System with image";
        let user_msg = "User with image request";
        let image_b64 = "fake_image_data";
        let request = LanguageModelBatchAPIRequest::new_with_image(model.clone(), idx, system_msg, user_msg, image_b64);
        debug!("Constructed request with image: {:?}", request);

        pretty_assert_eq!(request.custom_id().to_string(), "request-3");
        match request.method {
            HttpMethod::Post => trace!("Method is POST as expected"),
            _ => panic!("Expected POST method"),
        }
        match request.url {
            LanguageModelApiUrl::ChatCompletions => trace!("URL is ChatCompletions as expected"),
        }
        pretty_assert_eq!(
            request.body.messages().len(),
            2,
            "Should have system + user-with-image messages"
        );
        trace!("===== END TEST: new_with_image_produces_correct_fields =====");
    }

    #[traced_test]
    fn requests_from_query_strings_creates_requests_for_each_query() {
        trace!("===== BEGIN TEST: requests_from_query_strings_creates_requests_for_each_query =====");
        let system_message = "System greeting";
        let model = LanguageModelType::Gpt4o;
        let queries = vec!["Hello".to_string(), "World".to_string(), "Third".to_string()];
        let requests = LanguageModelBatchAPIRequest::requests_from_query_strings(system_message, model.clone(), &queries);
        debug!("Constructed requests: {:?}", requests);

        pretty_assert_eq!(
            requests.len(),
            queries.len(),
            "Number of requests should match number of queries"
        );
        for (idx, req) in requests.iter().enumerate() {
            let expected_custom_id = format!("request-{}", idx);
            pretty_assert_eq!(req.custom_id().to_string(), expected_custom_id);
            match req.url {
                LanguageModelApiUrl::ChatCompletions => (),
            }
        }
        trace!("===== END TEST: requests_from_query_strings_creates_requests_for_each_query =====");
    }

    #[traced_test]
    fn display_formats_as_json() {
        trace!("===== BEGIN TEST: display_formats_as_json =====");
        let request = LanguageModelBatchAPIRequest::mock("test_display");
        let displayed = format!("{}", request);
        debug!("Display output: {}", displayed);

        // Just ensure it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&displayed)
            .expect("Display output should be valid JSON");
        debug!("Parsed JSON: {:?}", parsed);
        assert!(parsed.is_object(), "Top-level display output should be an object");
        trace!("===== END TEST: display_formats_as_json =====");
    }

    #[traced_test]
    fn into_batch_request_input_sets_expected_fields() {
        trace!("===== BEGIN TEST: into_batch_request_input_sets_expected_fields =====");
        let request = LanguageModelBatchAPIRequest::mock("test_conversion");
        let converted: BatchRequestInput = request.clone().into();
        debug!("Converted BatchRequestInput: {:?}", converted);

        pretty_assert_eq!(
            converted.custom_id,
            request.custom_id().to_string(),
            "Custom ID should match"
        );
        pretty_assert_eq!(
            converted.method,
            BatchRequestInputMethod::POST,
            "HTTP method should be POST"
        );
        pretty_assert_eq!(
            converted.url,
            BatchEndpoint::V1ChatCompletions,
            "URL should be V1ChatCompletions"
        );
        assert!(
            converted.body.is_some(),
            "Body should be present in the conversion"
        );
        trace!("===== END TEST: into_batch_request_input_sets_expected_fields =====");
    }

    #[traced_test]
    fn create_batch_input_file_writes_valid_json_lines() {
        trace!("===== BEGIN TEST: create_batch_input_file_writes_valid_json_lines =====");
        let requests = vec![
            LanguageModelBatchAPIRequest::mock("id-1"),
            LanguageModelBatchAPIRequest::mock("id-2"),
        ];
        let temp_dir = std::env::temp_dir();
        let output_file = temp_dir.join("test_batch_input_file.json");
        debug!("Temporary output file: {:?}", output_file);

        let result = create_batch_input_file(&requests, &output_file);
        assert!(result.is_ok(), "create_batch_input_file should succeed");

        let contents = std::fs::read_to_string(&output_file)
            .expect("Failed to read the output file");
        debug!("File contents:\n{}", contents);
        let lines: Vec<&str> = contents.trim().split('\n').collect();
        pretty_assert_eq!(lines.len(), 2, "Should have exactly 2 lines for 2 requests");

        for (i, line) in lines.iter().enumerate() {
            let parsed: serde_json::Value = serde_json::from_str(line)
                .expect("Line should be valid JSON");
            assert!(
                parsed.is_object(),
                "Each line should represent a JSON object"
            );
            let custom_id = parsed.get("custom_id")
                .and_then(|v| v.as_str())
                .unwrap_or("<missing>");
            debug!("Parsed line {} custom_id={}", i, custom_id);
            assert!(
                custom_id.contains(&format!("id-{}", i+1)),
                "Expected custom_id to match 'id-<i+1>'"
            );
        }

        // Clean up
        if let Err(err) = std::fs::remove_file(&output_file) {
            warn!("Failed to remove temp file: {:?}", err);
        }

        trace!("===== END TEST: create_batch_input_file_writes_valid_json_lines =====");
    }
}
