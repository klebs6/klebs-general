crate::ix!();

/// Body details of the API request.
#[derive(Debug, Serialize, Deserialize)]
pub struct GptRequestBody {

    /// Model used for the request.
    #[serde(with = "model_type")]
    model: GptModelType,

    /// Array of messages exchanged in the request.
    messages: Vec<GptMessage>,

    /// Maximum number of tokens to be used by the model.
    max_tokens: u32,
}

impl GptRequestBody {

    pub fn default_max_tokens() -> u32 {
        //1024 
        8192
    }

    pub fn default_max_tokens_given_image(_image_b64: &str) -> u32 {
        //TODO: is this the right value?
        2048
    }

    pub fn new_basic(model: GptModelType, system_message: &str, user_message: &str) -> Self {
        Self {
            model,
            messages: vec![
                GptMessage::system_message(system_message),
                GptMessage::user_message(user_message),
            ],
            max_tokens: Self::default_max_tokens(),
        }
    }

    pub fn new_with_image(model: GptModelType, system_message: &str, user_message: &str, image_b64: &str) -> Self {
        Self {
            model,
            messages: vec![
                GptMessage::system_message(system_message),
                GptMessage::user_message_with_image(user_message,image_b64),
            ],
            max_tokens: Self::default_max_tokens_given_image(image_b64),
        }
    }
}
