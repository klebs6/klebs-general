// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum UuidParseError {
        UuidError(uuid::Error),

        #[cmp_neq]
        IoError(std::io::Error),
    }
}
