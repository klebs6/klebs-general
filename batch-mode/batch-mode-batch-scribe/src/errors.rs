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
        OpenAIError(OpenAIError),
        IOError(std::io::Error),
        TokenizerError(TokenizerError),
        ParseTokenDescriptionLineError(ParseTokenDescriptionLineError),
        SerdeJsonError(serde_json::Error),
    }
}

impl fmt::Display for BatchInputCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchInputCreationError::IOError(err) => write!(f, "IO error occurred: {}", err),
            BatchInputCreationError::SerdeJsonError(err) => write!(f, "JSON serialization error: {}", err),
        }
    }
}
