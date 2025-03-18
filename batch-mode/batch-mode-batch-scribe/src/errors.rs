// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchInputCreationError {
        IOError(std::io::Error),
        SerdeJsonError(serde_json::Error),
    }

    pub enum ParseTokenDescriptionLineError {
        MissingToken,
        MissingDescription,
    }

    pub enum TokenizerError {
        TokenizerError(String),
    }

    pub enum LanguageModelBatchCreationError {
        #[display("attempting to construct a trivially small batch of size {len}. are you sure you want to do this?")]
        TrivialBatchSizeBlocked {
            len: usize,
        },
        OpenAIError(OpenAIError),
        IOError(std::io::Error),
        TokenizerError(TokenizerError),
        ParseTokenDescriptionLineError(ParseTokenDescriptionLineError),
        SerdeJsonError(serde_json::Error),
    }
}
