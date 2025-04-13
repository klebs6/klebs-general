// ---------------- [ File: workspacer-file-filter/src/workspacer_file_filter_error.rs ]
crate::ix!();

error_tree!{
    pub enum AiFileFilterError {

        #[display("IoError: {context} => {io_error}")]
        IoError {
            io_error: std::sync::Arc<std::io::Error>,
            context:  String,
        },

        #[display("An error occurred in the cargo toml layer: {0}")]
        CargoTomlError(CargoTomlError),

        #[display("Error from the language model workflow: {0}")]
        LanguageModelBatchWorkflowError(LanguageModelBatchWorkflowError),

        // Add these two to allow `?` from `WorkspacerDir::ensure_subdir_exists` 
        // and `BatchWorkspace::new_in`
        WorkspacerFallbackError(WorkspacerFallbackError),
        BatchWorkspaceError(BatchWorkspaceError),

        #[display("Generic fallback error in AiFileFilter")]
        GenericError,
    }
}


/// A small helper to map `AiFileFilterError` => `WorkspaceError`.
/// You might prefer to define `impl From<AiFileFilterError> for WorkspaceError`
/// in your code.  This is just a local approach.
pub fn map_filter_error_into_workspace_error(e: AiFileFilterError) -> WorkspaceError {
    use workspacer_errors::ReadmeWriteError; // or your custom error

    match e {
        // If you have direct equivalents, map them more precisely:
        AiFileFilterError::IoError{io_error, context} => {
            WorkspaceError::IoError {
                io_error,
                context,
            }
        }
        AiFileFilterError::LanguageModelBatchWorkflowError(lm_err) => {
            // wrap in e.g. `WorkspaceError::ReadmeWriteError(...)` or something relevant
            let dummy = ReadmeWriteError::AiReadmeWriterError; 
            warn!("Mapping AiFileFilterError::LanguageModelBatchWorkflowError => dummy readme error variant");
            WorkspaceError::ReadmeWriteError(dummy)
        }
        AiFileFilterError::CargoTomlError(e2) => {
            WorkspaceError::InvalidCargoToml(e2)
        }
        AiFileFilterError::WorkspacerFallbackError(e2) => {
            // Possibly embed in a `WorkspaceError::CrateError(...)`, or a new variant
            let message = format!("Workspacer fallback error: {:?}", e2);
            WorkspaceError::IoError {
                io_error: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, message.clone())),
                context: message,
            }
        }
        AiFileFilterError::BatchWorkspaceError(e2) => {
            // Possibly embed in `WorkspaceError::MultipleErrors(...)` or similar
            error!("Mapping AiFileFilterError::BatchWorkspaceError => returning cargo workspace error");
            WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: PathBuf::from("unknown"),
            }
        }
        AiFileFilterError::GenericError => {
            // fallback
            WorkspaceError::ReadmeWriteError(ReadmeWriteError::AiReadmeWriterError)
        }
    }
}
