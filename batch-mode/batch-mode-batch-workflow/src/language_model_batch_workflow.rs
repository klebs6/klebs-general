// ---------------- [ File: batch-mode-batch-workflow/src/language_model_batch_workflow.rs ]
crate::ix!();

/// Two new traits that users must implement:
/// 1) `ComputeSystemMessage` to provide a static or dynamic system message.
/// 2) `ComputeLanguageModelCoreQuery` to build requests for each seed item.
///
/// These traits are now required components of the overall workflow.
/// We define them in the same `batch_mode_batch_workflow` (or relevant) crate
/// so that the derive macro can reference them.
pub trait ComputeSystemMessage {
    /// Returns the system message to be applied to all requests.
    fn system_message() -> String;
}

pub trait ComputeLanguageModelCoreQuery {
    /// The seed item type (e.g., AiTomlWriterRequest).
    type Seed: HasAssociatedOutputName + Named;

    /// Builds a single language model API request for a given seed item.
    /// The macro will call this once per seed item inside `compute_language_model_requests()`.
    fn compute_language_model_core_query(
        &self,
        input: &Self::Seed
    ) -> String;
}

#[async_trait]
pub trait FinishProcessingUncompletedBatches {
    type Error;

    /// Possibly complete or discard partial data from prior
    /// runs.
    ///
    async fn finish_processing_uncompleted_batches(
        &self,
        expected_content_type: &ExpectedContentType
    ) -> Result<(), Self::Error>;
}

/// This is the trait we will typically need to implement manually
pub trait ComputeLanguageModelRequests {

    type Seed: HasAssociatedOutputName + Send + Sync;

    /// Identify which new items need to be processed and
    /// build the requests.
    ///
    fn compute_language_model_requests(
        &self,
        model:            &LanguageModelType,
        input_tokens:     &[Self::Seed]
    ) -> Vec<LanguageModelBatchAPIRequest>;
}

#[async_trait]
pub trait ProcessBatchRequests {

    type Error;

    /// Process each batch, writing it to disk or sending it
    /// to a remote server.
    ///
    async fn process_batch_requests(
        &self,
        batch_requests:        &[LanguageModelBatchAPIRequest],
        expected_content_type: &ExpectedContentType,
    ) -> Result<(), Self::Error>;
}

/// Trait describing a more general “batch workflow”
/// specialized to GPT expansions.
///
/// This approach can unify:
/// - Reconciling partial/incomplete state,
/// - Computing new requests,
/// - Chunking them,
/// - Sending them to a remote server,
/// - Handling the results.
#[async_trait]
pub trait LanguageModelBatchWorkflow<E: From<LanguageModelBatchCreationError>>: 
    FinishProcessingUncompletedBatches<Error = E>
    + ComputeLanguageModelRequests
    + ProcessBatchRequests<Error = E>
{
    const REQUESTS_PER_BATCH: usize = 80;

    async fn plant_seed_and_wait(
        &mut self,
        input_tokens: &[<Self as ComputeLanguageModelRequests>::Seed]
    ) -> Result<(), E>;

    /// High-level method that ties it all together.
    /// Fixes the mismatch by enumerating chunk-slices properly and passing
    /// `&[LanguageModelBatchAPIRequest]` to `process_batch_requests`.
    async fn execute_language_model_batch_workflow(
        &mut self,
        model:                 LanguageModelType,
        expected_content_type: ExpectedContentType,
        input_tokens:          &[<Self as ComputeLanguageModelRequests>::Seed]
    ) -> Result<(), E>
    {
        info!("Beginning full batch workflow execution");

        self.finish_processing_uncompleted_batches(&expected_content_type).await?;

        let requests: Vec<_> = self.compute_language_model_requests(&model, input_tokens);

        // `construct_batches` presumably returns something like an iterator of chunks.
        let enumerated_batches = construct_batches(&requests, Self::REQUESTS_PER_BATCH, false)?;

        // Enumerate so we have (batch_idx, chunk_of_requests).
        for (batch_idx, batch_requests) in enumerated_batches {
            info!("Processing batch #{}", batch_idx);
            // Here, `batch_requests` is a `&[LanguageModelBatchAPIRequest]`,
            // matching the expected parameter type in `process_batch_requests`.
            self.process_batch_requests(batch_requests, &expected_content_type).await?;
        }

        Ok(())
    }
}

/// This new trait is used to gather the final AI expansions from disk, 
/// matching each seed item to its parsed output JSON.
#[async_trait]
pub trait LanguageModelBatchWorkflowGatherResults {
    type Error;
    type Seed: HasAssociatedOutputName + Clone + Named;
    type Output: LoadFromFile<Error = SaveLoadError>;

    /// Gathers AI-generated JSON outputs for the given seeds in the same order, 
    /// returning `(Seed, Output)` pairs.
    async fn gather_results(
        &self,
        seeds: &[Self::Seed]
    ) -> Result<Vec<(Self::Seed, Self::Output)>, Self::Error>;
}
