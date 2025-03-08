crate::ix!();

#[async_trait]
impl DownloadOutputFile for BatchFileTriple {

    async fn download_output_file(
        &mut self,
        client: &OpenAIClientHandle,

    ) -> Result<(), BatchDownloadError> {

        info!("downloading batch output file");

        if self.output().is_some() {
            return Err(BatchDownloadError::OutputFileAlreadyExists { triple: self.clone() });
        }

        let metadata_filename = self.metadata_filename_which_maybe_does_not_yet_exist();
        let metadata          = BatchMetadata::load_from_file(&metadata_filename).await?;
        let output_file_id    = metadata.output_file_id()?;

        // Download the output file content
        let file_content = client.file_content(output_file_id).await?;

        // Write the content to the output file
        let output_path = self.output_filename_which_maybe_does_not_yet_exist().clone();

        assert!(!output_path.exists());

        std::fs::write(&output_path, file_content)?;

        // Update the triple with the output file path
        self.set_output_path(Some(output_path));

        Ok(())
    }
}
