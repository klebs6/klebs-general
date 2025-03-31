// ---------------- [ File: batch-mode-batch-workflow/src/error.rs ]
crate::ix!();

error_tree!{
    pub enum LanguageModelBatchWorkflowError {
        BatchWorkspaceError(BatchWorkspaceError),
        LanguageModelBatchCreationError(LanguageModelBatchCreationError),
        BatchReconciliationError(BatchReconciliationError),
        BatchDownloadError(BatchDownloadError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        BatchMetadataError(BatchMetadataError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        BatchValidationError(BatchValidationError),
        FileMoveError(FileMoveError),
        OpenAIClientError(OpenAIClientError),
        BatchInputCreationError(BatchInputCreationError),
        BatchProcessingError(BatchProcessingError),
        JsonParseError(JsonParseError),
        IOError(std::io::Error),
        SerdeJsonError(serde_json::Error),
        SaveLoad(SaveLoadError),
    }
}
