// ---------------- [ File: src/check_and_download_output_and_error_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckForAndDownloadOutputAndErrorOnline<E> for BatchFileTriple 
where BatchDownloadError: From<E>
{
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,

    ) -> Result<(), BatchDownloadError> {

        loop {
            match self.check_batch_status_online(client).await {
                Ok(status) => {
                    info!("batch online status: {:#?}", status);

                    // Download files if available
                    if status.output_file_available() {
                        self.download_output_file(client).await?;
                    }
                    if status.error_file_available() {
                        self.download_error_file(client).await?;
                    }
                    return Ok(());
                }
                Err(BatchDownloadError::BatchStillProcessing { batch_id }) => {
                    // Batch is still processing; decide whether to wait or exit
                    info!("Batch {} is still processing.", batch_id);
                    client.wait_for_batch_completion(&batch_id).await?;
                }
                Err(e) => {
                    // Other errors
                    return Err(e);
                }
            }
        }
    }
}
