// ---------------- [ File: src/check_batch_status_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckBatchStatusOnline<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
      + From<OpenAIClientError>
      + From<BatchMetadataError>
      + From<std::io::Error>,
{
    async fn check_batch_status_online(
        &self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<BatchOnlineStatus, E> {
        info!("checking batch status online");

        let metadata_filename = self.metadata_filename_which_maybe_does_not_yet_exist();
        let mut metadata = BatchMetadata::load_from_file(&metadata_filename).await?;
        let batch_id = metadata.batch_id().to_string();

        // pass &batch_id (coerces to &str)
        let batch = client.retrieve_batch(&batch_id).await?;

        match batch.status {
            BatchStatus::Completed => {
                metadata.set_output_file_id(batch.output_file_id.clone());
                metadata.set_error_file_id(batch.error_file_id.clone());
                metadata.save_to_file(&metadata_filename).await?;

                Ok(BatchOnlineStatus::from(&batch))
            }
            BatchStatus::Failed => {
                Err(BatchDownloadError::BatchFailed { batch_id }.into())
            }
            BatchStatus::Validating
            | BatchStatus::InProgress
            | BatchStatus::Finalizing => {
                // If you'd rather block until it's done, do:
                //   info!("batch is still processing; waiting for completion...");
                //   client.wait_for_batch_completion(&batch_id).await?;
                //   let new_batch = client.retrieve_batch(&batch_id).await?;
                //   if new_batch.status == BatchStatus::Completed { ... } else { ... }
                //
                // Otherwise, just return "still processing" as an error:
                Err(BatchDownloadError::BatchStillProcessing { batch_id }.into())
            }
            _ => {
                Err(BatchDownloadError::UnknownBatchStatus {
                    batch_id,
                    status: batch.status.clone(),
                }.into())
            }
        }
    }
}
