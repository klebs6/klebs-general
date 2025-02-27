// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum BatchProcessingError {
        BatchOutputProcessingError(BatchOutputProcessingError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        ReconciliationError(BatchReconciliationError),
        OpenAIClientError(OpenAIClientError),
        BatchMetadataError(BatchMetadataError),
        BatchDownloadError(BatchDownloadError),
        JsonParseError(JsonParseError),
        ReconciliationFailed { index: BatchIndex },
        EmptyBatchTriple { index: BatchIndex },
    }
}
