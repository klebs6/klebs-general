crate::ix!();

error_tree!{

    pub enum ParseTokenDescriptionLineError {
        MissingToken,
        MissingDescription,
    }

    pub enum TokenizerError {
        TokenizerError(String),
    }

    pub enum GptBatchCreationError {
        OpenAIError(OpenAIError),
        IOError(std::io::Error),
        TokenizerError(TokenizerError),
        ParseTokenDescriptionLineError(ParseTokenDescriptionLineError),
        SerdeJsonError(serde_json::Error),
    }
}
