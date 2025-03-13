crate::ix!();

error_tree!{
    pub enum LanguageModelBatchWorkflowError {
        BatchWorkspaceError(BatchWorkspaceError),
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
