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
#[derive(Debug, Serialize, Deserialize)]
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
    pub fn custom_id(&self) -> &CustomRequestId {
        &self.custom_id
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
