crate::ix!();

error_tree!{

    pub enum ErrorWritingBatchExpansionErrorFile {
        IoError(std::io::Error),
    }

    pub enum TokenExpanderError {
        BatchError(BatchError),
        BatchWorkspaceError(BatchWorkspaceError),
        BatchSuccessResponseHandlingError(BatchSuccessResponseHandlingError),
        TokenParseError(TokenParseError),
        SaveLoadError(SaveLoadError),
        OpenAIError(OpenAIError),
        UuidError(uuid::Error),
    }

    pub enum BatchError {
        FileMoveError(FileMoveError),
        CreationError(BatchCreationError),
        MetadataError(BatchMetadataError),
        ReconciliationError(BatchReconciliationError),
        ProcessingError(BatchProcessingError),
        DownloadError(BatchDownloadError),
        BatchValidationError(BatchValidationError),
    }

    pub enum BatchCreationError {
        InputCreationError(BatchInputCreationError),
        // Other batch creation errors
    }
}

