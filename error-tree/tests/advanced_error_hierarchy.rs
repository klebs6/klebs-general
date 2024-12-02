#![allow(dead_code)]
#![allow(unused_variables)]

use error_tree::*;
use serde_json;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use uuid;

// Mock types used in the error hierarchy
type BatchIndex = usize;
type CustomRequestId = usize;
type BatchStatus = String;
type BatchFileTripleReconciliationOperation = String;
type BatchFileTriple = String;

// Placeholder for OpenAI-related errors
#[derive(Debug)]
pub struct OpenAIError;

#[derive(Debug)]
pub struct OpenAIApiError;

#[derive(Debug)]
pub struct BatchInputCreationError;

// Now, define the error hierarchy
error_tree!{
    pub enum OpenAIClientError {
        OpenAIError(OpenAIError),
        ApiError(OpenAIApiError),
    }

    pub enum WorkspaceError {
        #[display("No existing batch file triple at the given index {index}")]
        NoBatchFileTripleAtIndex { index: BatchIndex },
        IoError(std::io::Error),
        ParseError(ParseError),
    }

    pub enum BatchOutputProcessingError {
        ErrorSavingFailedBatchEntries(ErrorSavingFailedBatchEntries),
        ParseError(ParseError),
        IoError(std::io::Error),
        SerializationError(serde_json::Error),
    }

    pub enum BatchErrorProcessingError {
        ParseError(ParseError),
    }

    pub enum ErrorSavingFailedBatchEntries {
        SerdeJsonError(serde_json::Error),
        IoError(std::io::Error),
    }

    pub enum ErrorWritingBatchExpansionErrorFile {
        IoError(std::io::Error),
    }

    pub enum FileMoveError {
        IoError(std::io::Error),
    }

    pub enum TokenExpanderError {
        BatchError(BatchError),
        WorkspaceError(WorkspaceError),
        BatchSuccessResponseHandlingError(BatchSuccessResponseHandlingError),
        ParseError(ParseError),
        FileError(FileError),
        OpenAIError(OpenAIError),
        UuidError(uuid::Error),
    }

    pub enum BatchValidationError {
        ParseError(ParseError),
        RequestIdsMismatch {
            index:      BatchIndex,
            input_ids:  Option<HashSet<CustomRequestId>>,
            output_ids: Option<HashSet<CustomRequestId>>,
            error_ids:  Option<HashSet<CustomRequestId>>,
        },
    }

    pub enum BatchError {
        CreationError(BatchCreationError),
        MetadataError(BatchMetadataError),
        //ReconciliationError(BatchReconciliationError),
        ProcessingError(BatchProcessingError),
        DownloadError(BatchDownloadError),
        BatchValidationError(BatchValidationError),
    }

    pub enum BatchCreationError {
        InputCreationError(BatchInputCreationError),
        // Other batch creation errors
    }

    pub enum BatchMetadataError {
        MissingOutputFileId,
        MissingErrorFileId,
        SerializationError(serde_json::Error),
        IoError(std::io::Error),
    }

    pub enum BatchReconciliationError {
        WorkspaceError(WorkspaceError),
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

    pub enum BatchProcessingError {
        ReconciliationError(BatchReconciliationError),
        OpenAIClientError(OpenAIClientError),
        BatchMetadataError(BatchMetadataError),
        BatchDownloadError(BatchDownloadError),
        ParseError(ParseError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        ReconciliationFailed { index: BatchIndex },
        EmptyBatchTriple { index: BatchIndex },
    }

    pub enum BatchDownloadError {
        BatchFailed             { batch_id: String },
        BatchStillProcessing    { batch_id: String },
        ErrorFileAlreadyExists  { triple: BatchFileTriple },
        OutputFileAlreadyExists { triple: BatchFileTriple },
        UnknownBatchStatus      { batch_id: String, status:   BatchStatus, },
        BatchMetadataError(BatchMetadataError),
        IoError(std::io::Error),
        OpenAIClientError(OpenAIClientError),
    }

    pub enum BatchSuccessResponseHandlingError {
        ParseError(ParseError),
        FileError(FileError),
    }

    pub enum ParseError {
        InvalidTokenName,
        InvalidContent,
        InvalidJson,
        InvalidLine(String),
        MissingTokenNameField,
        JsonParsingError(serde_json::Error),
        IoError(std::io::Error),
        UuidError(uuid::Error),
    }

    pub enum FileError {
        IoError(std::io::Error),
    }
}

#[test]
fn test_multiple_paths() {
    let io_error = io::Error::new(io::ErrorKind::Other, "An IO error occurred");

    // Attempt to convert directly to TokenExpanderError
    // This should fail to compile if the macro correctly avoids generating conflicting From implementations
    // Uncommenting the line below should cause a compilation error
    // let token_expander_error: TokenExpanderError = io_error.into(); // Should not compile
}

#[test]
fn test_error_tree_conversions() {
    // Create instances of low-level errors
    let io_error = io::Error::new(io::ErrorKind::Other, "An IO error occurred");

    let invalid_uuid = "invalid_uuid";
    let uuid_error = invalid_uuid.parse::<uuid::Uuid>().unwrap_err();

    let invalid_json = "invalid_json";
    let serde_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();

    // Attempt to convert them into higher-level errors
    let parse_error_from_io: ParseError = io_error.into();
    let parse_error_from_uuid: ParseError = uuid_error.into();
    let parse_error_from_serde: ParseError = serde_error.into();

    // Now, try to convert ParseError into higher-level errors
    let workspace_error: WorkspaceError = parse_error_from_io.into();
    let batch_output_processing_error: BatchOutputProcessingError = parse_error_from_serde.into();

    // Try converting BatchOutputProcessingError into BatchReconciliationError
    let batch_reconciliation_error: BatchReconciliationError = batch_output_processing_error.into();

    // Finally, attempt to convert BatchReconciliationError into TokenExpanderError
    let token_expander_error: TokenExpanderError = batch_reconciliation_error.into();

    // Print the errors to verify
    println!("WorkspaceError: {}", workspace_error);
    println!("TokenExpanderError: {}", token_expander_error);
}
