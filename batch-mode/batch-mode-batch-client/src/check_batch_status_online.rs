// ---------------- [ File: src/check_batch_status_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckBatchStatusOnline<E> for BatchFileTriple 
where BatchDownloadError: From<E>
{
    async fn check_batch_status_online(
        &self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<BatchOnlineStatus, BatchDownloadError> {

        info!("checking batch status online");

        // Load batch metadata to get the batch ID
        let metadata_filename = self.metadata_filename_which_maybe_does_not_yet_exist();
        let mut metadata      = BatchMetadata::load_from_file(&metadata_filename).await?;
        let batch_id          = metadata.batch_id().to_string();

        // Retrieve batch status from the API
        let batch = client.retrieve_batch(&batch_id).await?;

        match batch.status {
            BatchStatus::Completed => {

                // Update metadata with file IDs
                metadata.set_output_file_id(batch.output_file_id.clone());
                metadata.set_error_file_id(batch.error_file_id.clone());
                metadata.save_to_file(&metadata_filename).await?;

                Ok(BatchOnlineStatus::from(&batch))
            }
            BatchStatus::Failed => {
                Err(BatchDownloadError::BatchFailed {
                    batch_id,
                })
            }
            BatchStatus::Validating | BatchStatus::InProgress | BatchStatus::Finalizing => {
                // Batch is still processing
                Err(BatchDownloadError::BatchStillProcessing {
                    batch_id,
                })
            }
            _ => {
                // Handle other statuses if necessary
                Err(BatchDownloadError::UnknownBatchStatus {
                    batch_id,
                    status: batch.status.clone(),
                })
            }
        }
    }
}
