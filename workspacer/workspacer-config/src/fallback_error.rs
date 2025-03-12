crate::ix!();

/// A generic error type for fallback reading—adjust or merge into your own
/// error definitions as needed. In a real codebase, you might unify this with
/// `workspacer_errors::CargoTomlError` or define a dedicated error for config
/// fallback failures.
error_tree!{

    #[derive(Clone)]
    pub enum WorkspacerFallbackError {
        #[display("I/O error: {0}")]
        Io(Arc<std::io::Error>),

        #[display("Failed to parse config as TOML: {0}")]
        TomlParse(toml::de::Error),

        #[display("Missing .ws directory or config file")]
        MissingConfig,

        #[display("No home directory could be found, cannot perform global .ws operations")]
        NoHomeDirectory,
    }
}
