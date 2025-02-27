// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum SaveLoadError {
        IoError(std::io::Error),
        JsonParseError(JsonParseError),
        InvalidDirectory {
            dir: PathBuf,
        }
    }
}
