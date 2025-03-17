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

    #[derive(PartialEq)]
    pub enum JsonParseError {
        #[cmp_neq] 
        SerdeError(serde_json::Error),

        #[cmp_neq] 
        IoError(std::io::Error),

        JsonRepairError(JsonRepairError),
        InvalidJson,
    }
}
