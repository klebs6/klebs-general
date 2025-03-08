// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum TokenExpanderError {
        BatchError(BatchError),
        BatchWorkspaceError(BatchWorkspaceError),
        BatchSuccessResponseHandlingError(BatchSuccessResponseHandlingError),
        TokenParseError(TokenParseError),
        SaveLoadError(SaveLoadError),
        OpenAIError(OpenAIError),
        FileMoveError(FileMoveError),
        BatchProcessingError(BatchProcessingError),
        BatchInputCreationError(BatchInputCreationError),
        BatchReconciliationError(BatchReconciliationError),
        UuidError(uuid::Error),
    }
}
