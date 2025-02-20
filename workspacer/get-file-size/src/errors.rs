// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(Clone)]
    pub enum FileError {
        CreationError {
            io: Arc<io::Error>,
        },
        WriteError {
            io: Arc<io::Error>,
        },
        GetMetadataError {
            io: Arc<io::Error>,
        },
        OpenError {
            io: Arc<io::Error>,
        },
        GetNextLineError {
            io: Arc<io::Error>,
        },
    }
}
