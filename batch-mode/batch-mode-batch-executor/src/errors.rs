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

        #[display("BatchProcessingError: reconciliation failed. index={index:?}")]
        ReconciliationFailed { index: BatchIndex },

        #[display("BatchProcessingError: empty batch triple. index={index:?}")]
        EmptyBatchTriple { index: BatchIndex },
    }
}

impl From<BatchProcessingError> for MockBatchClientError {
    fn from(e: BatchProcessingError) -> Self {
        MockBatchClientError::BatchProcessingError
    }
}
