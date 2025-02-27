// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum TokenParseError {
        InvalidTokenName,
        InvalidTokenLine(String),
        MissingTokenNameField,

        #[cmp_neq]
        IoError(std::io::Error),
    }
}
