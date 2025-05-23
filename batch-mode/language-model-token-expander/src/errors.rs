// ---------------- [ File: language-model-token-expander/src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum TokenExpanderError {
        LanguageModelBatchCreationError(LanguageModelBatchCreationError),
        BatchDownloadError(BatchDownloadError),
        BatchError(BatchError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        BatchInputCreationError(BatchInputCreationError),
        BatchMetadataError(BatchMetadataError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        BatchProcessingError(BatchProcessingError),
        BatchReconciliationError(BatchReconciliationError),
        BatchSuccessResponseHandlingError(BatchSuccessResponseHandlingError),
        BatchValidationError(BatchValidationError),
        BatchWorkspaceError(BatchWorkspaceError),
        FileMoveError(FileMoveError),
        IOError(std::io::Error),
        OpenAIClientError(OpenAIClientError),
        SaveLoadError(SaveLoadError),
        TokenParseError(TokenParseError),
        UuidError(uuid::Error),
        JsonParseError(JsonParseError),
    }
}
