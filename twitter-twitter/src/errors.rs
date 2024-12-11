crate::ix!();

error_tree!{

    pub enum ConfigError {
        Default,
    }

    pub enum AuthError {
        Default,
    }

    pub enum OAuthError {
        Default,
    }

    pub enum TimeParseError {
        Default,
    }

    pub enum MessengerError {
        ConfigError(ConfigError),
        ConfigBuilderError(ConfigBuilderError),
        AuthError(AuthError),
        IoError(std::io::Error),
        HttpRequestError(reqwest::Error),
        SerdeError(serde_json::Error),
        OAuthError(OAuthError),
        TimeParseError(TimeParseError),
        UnknownError,
    }
}
