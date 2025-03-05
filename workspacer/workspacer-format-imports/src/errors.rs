// ---------------- [ File: workspacer-format-imports/src/errors.rs ]
crate::ix!();

/// A convenience error type for problems encountered while sorting/formatting imports.
error_tree!{
    pub enum SortAndFormatImportsError {
        IoError(Arc<std::io::Error>),
        RaApParseError {
            parse_errors: String,
        },
        RewriteError,
        Other,
    }
}
