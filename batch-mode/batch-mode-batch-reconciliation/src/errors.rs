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
        BatchDownloadError(BatchDownloadError),
        BatchValidationError(BatchValidationError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        FileMoveError(FileMoveError),
        OperationNotImplemented {
            operation: BatchFileTripleReconciliationOperation,
        },
        ReconciliationFailed {
            index:  BatchIndex,
            errors: Vec<(BatchFileTripleReconciliationOperation,BatchReconciliationError)>,
        },
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
