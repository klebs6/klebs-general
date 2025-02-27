// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum BatchMetadataError {
        MissingOutputFileId,
        MissingErrorFileId,
        SerializationError(serde_json::Error),
        IoError(std::io::Error),
    }
}
