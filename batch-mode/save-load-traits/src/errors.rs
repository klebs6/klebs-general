// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum SaveLoadError {
        IoError(std::io::Error),
        JsonParseError(JsonParseError),
        SerdeJsonError(serde_json::Error),

        #[display("SaveLoadError: {dir:?} is an InvalidDirectory")]
        InvalidDirectory {
            dir: PathBuf,
        }
    }
}
