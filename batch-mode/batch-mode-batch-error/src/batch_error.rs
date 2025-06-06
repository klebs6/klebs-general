// ---------------- [ File: batch-mode-batch-error/src/batch_error.rs ]
crate::ix!();

error_tree!{

    pub enum ErrorWritingBatchExpansionErrorFile {
        IoError(std::io::Error),
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
