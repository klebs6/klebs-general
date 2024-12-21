crate::ix!();

error_tree!{
    pub enum DownloadError {
        IoError(io::Error),
        HttpError(reqwest::Error),
        Md5ChecksumVerificationError(Md5ChecksumVerificationError),
    }

    pub enum Md5ChecksumVerificationError {
        ChecksumMismatch { 
            expected: String, 
            actual:   String 
        },
        IoError(io::Error),
    }
}
