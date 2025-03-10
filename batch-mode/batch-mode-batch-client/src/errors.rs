// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchDownloadError {

        #[display("BatchDownloadError: batch failed. batch_id={batch_id:?}")]
        BatchFailed             { batch_id: String },

        #[display("BatchDownloadError: batch still processing. batch_id={batch_id:?}")]
        BatchStillProcessing    { batch_id: String },

        #[display("BatchDownloadError: error file already exists. batch_triple={triple:?}")]
        ErrorFileAlreadyExists  { triple: BatchFileTriple },

        #[display("BatchDownloadError: output file already exists. batch_triple={triple:?}")]
        OutputFileAlreadyExists { triple: BatchFileTriple },

        #[display("BatchDownloadError: unknown batch status. batch_id={batch_id:?}, batch_status={status:?}")]
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
