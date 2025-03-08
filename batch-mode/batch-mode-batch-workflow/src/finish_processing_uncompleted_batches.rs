/// That Yogin who is freed from attachment and pride, who transcends all pairs of opposites such
/// as pleasure and pain, who never gives way to wrath or hate, who never speaks an untruth, who
/// though slandered or struck still shows friendship for the slanderer or the striker, who never
/// thinks of doing ill to others, who restrains these three, viz. speech, acts and mind, and who
/// behaves uniformly towards all creatures, succeeds in approaching Brahman (true self).
/// 
/// — The Mahabharata, Shanti Parva, Chapter CCXXXVI, 

crate::ix!();

#[async_trait]
pub trait FinishProcessingUncompletedBatches<E> {

    /// Possibly complete or discard partial data from prior
    /// runs.
    ///
    async fn finish_processing_uncompleted_batches(
        &self,
        expected_content_type: &ExpectedContentType
    ) -> Result<(), E>;
}

#[async_trait]
impl<T, E> FinishProcessingUncompletedBatches<E> for T
where
    T: Send + Sync + GetBatchWorkspace<E> + GetLanguageModelClient<E>,
    E: From<BatchReconciliationError>,
    BatchDownloadError: From<E>,
{
    async fn finish_processing_uncompleted_batches(
        &self,
        expected_content_type: &ExpectedContentType
    ) -> Result<(), E> 
    {
        info!("Finishing uncompleted batches if any remain.");

        let workspace             = self.workspace();
        let language_model_client = self.language_model_client();

        let mut batch_triples = workspace.gather_all_batch_triples().await?;
        info!("Reconciling unprocessed batch files in the work directory");

        // NOTICE: We pass the constants as function pointers:
        //   &PROCESS_OUTPUT_FILE_BRIDGE
        //   &PROCESS_ERROR_FILE_BRIDGE
        // 
        // Both have the needed signature. 
        // The compiler then sees them as 
        // `&for<'a> fn(...) -> Pin<Box<...+'a>>`.
        // 
        for triple in &mut batch_triples {
            triple
                .reconcile_unprocessed(
                    &*language_model_client,
                    expected_content_type,
                    &PROCESS_OUTPUT_FILE_BRIDGE,
                    &PROCESS_ERROR_FILE_BRIDGE,
                )
                .await?;
        }
        Ok(())
    }
}
