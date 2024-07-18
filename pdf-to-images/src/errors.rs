crate::ix!();

error_tree!{

    pub enum PdfAppError {
        CairoError(cairo::Error),
        CairoIoError(cairo::IoError),
        StdIoError(std::io::Error),
        EventLoopError(EventLoopError),
    }
}
