// ---------------- [ File: src/check_and_download_output_and_error_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckForAndDownloadOutputAndErrorOnline<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
      + From<OpenAIClientError>
      + From<BatchMetadataError>
      + From<std::io::Error>,
{
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E> {
        info!("Checking for and downloading output/error files if available.");

        // Just call check_batch_status_online:
        // If we are incomplete, or have a failure, that function returns an error.
        let status = self.check_batch_status_online(client).await?;

        // If completed, let's download whichever are available:
        info!("batch online status: {:?}", status);

        if status.output_file_available() {
            self.download_output_file(client).await?;
        }
        if status.error_file_available() {
            self.download_error_file(client).await?;
        }

        Ok(())
    }
}
