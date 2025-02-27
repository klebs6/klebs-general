// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(PartialEq)]
    pub enum ContentParseError {
        InvalidContent,

        #[cmp_neq]
        IoError(std::io::Error),
    }
}
