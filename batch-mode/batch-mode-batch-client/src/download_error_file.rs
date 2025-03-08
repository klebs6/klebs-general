// ---------------- [ File: src/download_error_file.rs ]
crate::ix!();

#[async_trait]
impl<E> DownloadErrorFile<E> for BatchFileTriple 
where BatchDownloadError: From<E>
{
    async fn download_error_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), BatchDownloadError> {

        info!("downloading batch error file");

        if self.error().is_some() {
            return Err(BatchDownloadError::ErrorFileAlreadyExists { triple: self.clone() });
        }

        let metadata_filename = self.metadata_filename_which_maybe_does_not_yet_exist();
        let metadata          = BatchMetadata::load_from_file(&metadata_filename).await?;
        let error_file_id     = metadata.error_file_id()?;

        // Download the error file content
        let file_content = client.file_content(error_file_id).await?;

        // Write the content to the error file
        let error_path = self.error_filename_which_maybe_does_not_yet_exist();

        assert!(!error_path.exists());

        std::fs::write(&error_path, file_content)?;

        // Update the triple with the error file path
        self.set_error_path(Some(error_path));

        Ok(())
    }
}
