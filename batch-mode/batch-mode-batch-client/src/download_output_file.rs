// ---------------- [ File: src/download_output_file.rs ]
crate::ix!();

#[async_trait]
impl<E> DownloadOutputFile<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
      + From<std::io::Error>
      + From<BatchMetadataError>
      + From<OpenAIClientError>,
{
    async fn download_output_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E> {
        info!("downloading batch output file");

        if self.output().is_some() {
            return Err(BatchDownloadError::OutputFileAlreadyExists {
                triple: self.clone(),
            }
            .into());
        }

        let metadata_filename = self.metadata_filename_which_maybe_does_not_yet_exist();
        let metadata = BatchMetadata::load_from_file(&metadata_filename).await?;
        let output_file_id = metadata.output_file_id()?;

        let file_content = client.file_content(output_file_id).await?;

        let output_path = self.output_filename_which_maybe_does_not_yet_exist().clone();
        assert!(!output_path.exists());

        std::fs::write(&output_path, file_content)?;
        self.set_output_path(Some(output_path));

        Ok(())
    }
}
