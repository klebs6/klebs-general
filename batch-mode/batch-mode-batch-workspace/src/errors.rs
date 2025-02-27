// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum BatchWorkspaceError {
        #[display("No existing batch file triple at the given index {index}")]
        NoBatchFileTripleAtIndex { index: BatchIndex },
        IoError(std::io::Error),
        JsonParseError(JsonParseError),
        UuidParseError(UuidParseError),
        SaveLoadError(SaveLoadError),
    }
}
