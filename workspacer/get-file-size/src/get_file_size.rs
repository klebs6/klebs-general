// ---------------- [ File: get-file-size/src/get_file_size.rs ]
// ---------------- [ File: get-file-size/src/get_file_size.rs ]
crate::ix!();

#[async_trait]
pub trait GetFileSize {

    async fn file_size(&self) -> Result<u64, FileError>;
}

#[async_trait]
impl<T> GetFileSize for T
where
    T: AsRef<Path> + Send + Sync,
{
    async fn file_size(&self) -> Result<u64, FileError> {
        Ok(tokio::fs::metadata(self.as_ref())
            .await
            .map_err(|e| FileError::GetMetadataError { io: e.into() })?
            .len())
    }
}

