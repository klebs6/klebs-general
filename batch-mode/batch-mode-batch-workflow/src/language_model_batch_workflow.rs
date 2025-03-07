crate::ix!();

/// Trait describing a more general “batch workflow” specialized to GPT expansions.
/// This approach can unify:
/// - Reconciling partial/incomplete state,
/// - Computing new requests,
/// - Chunking them,
/// - Sending them to a remote server,
/// - Handling the results.
#[async_trait]
pub trait LanguageModelBatchWorkflow {

    type Seed:  Send + Sync;
    type Error: Send + Sync;

    /// Possibly complete or discard partial data from prior runs.
    async fn maybe_finish_processing_uncompleted_batches(
        &self,
        expected_content_type: ExpectedContentType
    ) -> Result<(), Self::Error>;

    /// Identify which new items need to be processed and build the requests.
    fn compute_language_model_requests(
        &mut self,
        model: &LanguageModelType,
        input_tokens: &[Self::Seed]
    );

    /// Break requests into workable batches.
    fn construct_batches(
        &self
    ) -> Enumerate<Chunks<'_, LanguageModelBatchAPIRequest>>;

    /// Process each batch, writing it to disk or sending it to a remote server.
    async fn process_batch_requests(
        &self,
        batch_requests: &[LanguageModelBatchAPIRequest]
    ) -> Result<(), Self::Error>;

    /// High-level method that ties it all together:
    async fn execute_workflow(
        &mut self,
        model: &LanguageModelType,
        input_tokens: &[Self::Seed]
    ) -> Result<(), Self::Error>;
}
