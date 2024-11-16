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
pub struct GptBatchAPIRequest {

    /// Identifier for the custom request.
    custom_id: String,

    /// HTTP method used for the request.
    #[serde(with = "http_method")]
    method: HttpMethod,

    /// URL of the API endpoint.
    #[serde(with = "api_url")]
    url:  GptApiUrl,

    /// Body of the request.
    body: GptRequestBody,
}

impl From<GptBatchAPIRequest> for BatchRequestInput {

    fn from(request: GptBatchAPIRequest) -> Self {
        BatchRequestInput {
            custom_id: request.custom_id,
            method: BatchRequestInputMethod::POST,
            url: match request.url {
                GptApiUrl::ChatCompletions => BatchEndpoint::V1ChatCompletions,
            },
            body: Some(serde_json::to_value(&request.body).unwrap()),
        }
    }
}

pub fn create_batch_input_file(
    requests:             &[GptBatchAPIRequest],
    batch_input_filename: impl AsRef<Path>,

) -> Result<(), Box<dyn std::error::Error>> {

    use std::io::{BufWriter,Write};
    use std::fs::File;

    let file = File::create(batch_input_filename.as_ref())?;
    let mut writer = BufWriter::new(file);

    for request in requests {
        let batch_input = BatchRequestInput {
            custom_id: request.custom_id.clone(),
            method: match request.method {
                HttpMethod::Post => BatchRequestInputMethod::POST,
                _ => unimplemented!("Only POST method is supported"),
            },
            url: match request.url {
                GptApiUrl::ChatCompletions => BatchEndpoint::V1ChatCompletions,
                // Handle other endpoints if necessary
            },
            body: Some(serde_json::to_value(&request.body)?),
        };
        let line = serde_json::to_string(&batch_input)?;
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

impl GptBatchAPIRequest {

    pub fn requests_from_query_strings(system_message: &str, model: GptModelType, queries: &[String]) -> Vec<Self> {
        queries.iter().enumerate().map(|(idx,query)| Self::new_basic(model,idx,system_message,&query)).collect()
    }

    pub fn new_basic(model: GptModelType, idx: usize, system_message: &str, user_message: &str) -> Self {
        Self {
            custom_id: Self::custom_id_for_idx(idx),
            method:    HttpMethod::Post,
            url:       GptApiUrl::ChatCompletions,
            body:      GptRequestBody::new_basic(model,system_message,user_message),
        }
    }

    pub fn new_with_image(model: GptModelType, idx: usize, system_message: &str, user_message: &str, image_b64: &str) -> Self {
        Self {
            custom_id: Self::custom_id_for_idx(idx),
            method:    HttpMethod::Post,
            url:       GptApiUrl::ChatCompletions,
            body:      GptRequestBody::new_with_image(model,system_message,user_message,image_b64),
        }
    }
}

impl Display for GptBatchAPIRequest {
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

impl GptBatchAPIRequest {

    pub(crate) fn custom_id_for_idx(idx: usize) -> String {
        format!("request-{}",idx)
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

    pub fn serialize<S>(value: &GptApiUrl, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<GptApiUrl, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "/v1/chat/completions" => Ok(GptApiUrl::ChatCompletions),
            _ => Err(serde::de::Error::custom("unknown URL")),
        }
    }
}
