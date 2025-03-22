// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchErrorProcessingError {
        JsonParseError(JsonParseError),
        IoError(std::io::Error),
        MissingFilePath,
    }

    pub enum FileMoveError {
        IoError(std::io::Error),
    }

    pub enum BatchValidationError {
        JsonParseError(JsonParseError),

        #[display("BatchValidationError: Request IDs mismatch. {index:#?} {input_ids:#?} {output_ids:#?} {error_ids:#?}")]
        RequestIdsMismatch {
            index:      BatchIndex,
            input_ids:  Option<HashSet<CustomRequestId>>,
            output_ids: Option<HashSet<CustomRequestId>>,
            error_ids:  Option<HashSet<CustomRequestId>>,
        },
    }
}
