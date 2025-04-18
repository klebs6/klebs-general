// ---------------- [ File: batch-mode-batch-reconciliation/src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchOutputProcessingError {
        ErrorSavingFailedBatchEntries(ErrorSavingFailedBatchEntries),
        JsonParseError(JsonParseError),
        IoError(std::io::Error),
        SerializationError(serde_json::Error),
        MissingFilePath,
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
        SerdeJsonError(serde_json::Error),
        TokenParseError(TokenParseError),
        SaveLoadError(SaveLoadError),
        IoError(std::io::Error),
    }
}

impl From<BatchReconciliationError> for MockBatchClientError {
    fn from(e: BatchReconciliationError) -> Self {
        MockBatchClientError::BatchReconciliationError { index: e.index().expect("todo: checkme") }
    }
}

impl From<BatchOutputProcessingError> for MockBatchClientError {
    fn from(_e: BatchOutputProcessingError) -> Self {
        MockBatchClientError::BatchOutputProcessingError
    }
}

impl BatchReconciliationError {
    pub fn index(&self) -> Option<BatchIndex> {
        match self {
            BatchReconciliationError::ReconciliationFailed { index } => {
                Some(index.clone())
            }

            BatchReconciliationError::MissingBatchInputFileButOthersExist { index, output: _, error: _ } => {
                Some(index.clone())
            }
            _ => None
        }
    }
}
