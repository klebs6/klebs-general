// ---------------- [ File: src/language_model_api_url.rs ]
crate::ix!();

/// Enumeration of API URLs.
#[derive(Clone,Debug, Serialize, Deserialize)]
pub enum LanguageModelApiUrl {

    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
}

impl fmt::Display for LanguageModelApiUrl {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageModelApiUrl::ChatCompletions => write!(f, "/v1/chat/completions"),
        }
    }
}
