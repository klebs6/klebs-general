// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchOutputProcessingError {
        ErrorSavingFailedBatchEntries(ErrorSavingFailedBatchEntries),
        JsonParseError(JsonParseError),
        IoError(std::io::Error),
        SerializationError(serde_json::Error),
    }

    pub enum ErrorSavingFailedBatchEntries {
        SerdeJsonError(serde_json::Error),
        IoError(std::io::Error),
    }

    pub enum BatchReconciliationError {
        BatchWorkspaceError(BatchWorkspaceError),
        OpenAIClientError(OpenAIClientError),
        BatchMetadataError(BatchMetadataError),
        IOError(std::io::Error),
        BatchDownloadError(BatchDownloadError),
        BatchValidationError(BatchValidationError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        FileMoveError(FileMoveError),

        #[display("BatchReconciliationError: operation not implemented. operation={operation:?}")]
        OperationNotImplemented {
            operation: BatchFileTripleReconciliationOperation,
        },

        #[display("BatchReconciliationError: reconciliation failed. index={index:?}")]
        ReconciliationFailed {
            index:  BatchIndex,
            //errors: Vec<(BatchFileTripleReconciliationOperation,BatchReconciliationError)>,
        },

        #[display("BatchReconciliationError: missing input file. index={index:?}, output={output:?}, error={error:?}")]
        MissingBatchInputFileButOthersExist {
            index:  BatchIndex,
            output: Option<PathBuf>,
            error:  Option<PathBuf>,
        },
    }

    pub enum BatchSuccessResponseHandlingError {
        UuidParseError(UuidParseError),
        JsonParseError(JsonParseError),
        TokenParseError(TokenParseError),
        SaveLoadError(SaveLoadError),
        IoError(std::io::Error),
    }
}
