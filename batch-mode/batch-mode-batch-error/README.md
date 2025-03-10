this crate contains common errors for the batch-mode system.

```rust
// ---------------- [ File: src/batch_error.rs ]
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
```
