crate::ix!();

/// Enumeration of API URLs.
#[derive(Debug, Serialize, Deserialize)]
pub enum GptApiUrl {

    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
}

impl fmt::Display for GptApiUrl {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GptApiUrl::ChatCompletions => write!(f, "/v1/chat/completions"),
        }
    }
}
