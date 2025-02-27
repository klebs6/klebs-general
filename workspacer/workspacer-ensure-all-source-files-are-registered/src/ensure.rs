crate::ix!();

#[async_trait]
pub trait EnsureAllSourceFilesAreRegistered {
    type Error;
    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error>;
}
