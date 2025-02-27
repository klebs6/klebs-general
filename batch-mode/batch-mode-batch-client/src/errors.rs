// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

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

    pub enum OpenAIClientError {
        OpenAIError(OpenAIError),
        ApiError(OpenAIApiError),
    }
}
