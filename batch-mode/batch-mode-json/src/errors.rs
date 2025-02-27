// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum JsonParseError {

        JsonRepairError(JsonRepairError),

        InvalidJson,

        #[cmp_neq]
        SerdeError(serde_json::Error),

        #[cmp_neq]
        IoError(std::io::Error),
    }
}
