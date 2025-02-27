// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchErrorProcessingError {
        JsonParseError(JsonParseError),
    }

    pub enum FileMoveError {
        IoError(std::io::Error),
    }

    pub enum BatchValidationError {
        JsonParseError(JsonParseError),
        RequestIdsMismatch {
            index:      BatchIndex,
            input_ids:  Option<HashSet<CustomRequestId>>,
            output_ids: Option<HashSet<CustomRequestId>>,
            error_ids:  Option<HashSet<CustomRequestId>>,
        },
    }
}
