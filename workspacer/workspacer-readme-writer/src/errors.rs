// ---------------- [ File: workspacer-readme-writer/src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum AiReadmeWriterError {
        LanguageModelBatchWorkflowError(LanguageModelBatchWorkflowError),
        BatchWorkspaceError(BatchWorkspaceError),
        WorkspacerFallbackError(WorkspacerFallbackError),
        CrateError(CrateError),
        WorkspaceError(WorkspaceError),
        CargoTomlError(CargoTomlError),
        #[display("ReadmeWriteError: {0}")]
        ReadmeWriteError(ReadmeWriteError),
    }
}

impl From<AiReadmeWriterError> for WorkspaceError {
    fn from(err: AiReadmeWriterError) -> Self {
        // Decide how you want to embed that error in a `WorkspaceError`.
        // For example, you can wrap it in `WorkspaceError::ReadmeWriteError(...)` 
        // or a generic variant. If you have no direct variant for it, you can do 
        // a new variant or wrap it in `WorkspaceError::CrateError(...)`, etc.
        //
        // Example:
        match err {
            AiReadmeWriterError::CrateError(ce) => WorkspaceError::CrateError(ce),
            AiReadmeWriterError::WorkspaceError(we) => we,
            other => {
                // fallback:
                WorkspaceError::ReadmeWriteError(ReadmeWriteError::AiReadmeWriterError)
            }
        }
    }
}
