crate::ix!();

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

pub trait ComputeLanguageModelRequests {

    type Seed;

    /// Identify which new items need to be processed and
    /// build the requests.
    ///
    fn compute_language_model_requests(
        &mut self,
        model:        &LanguageModelType,
        input_tokens: &[Self::Seed]
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
pub trait LanguageModelBatchWorkflow
: FinishProcessingUncompletedBatches 
+ ComputeLanguageModelRequests
+ ProcessBatchRequests
{
    const REQUESTS_PER_BATCH: usize = 80;

    /// High-level method that ties it all together:
    async fn execute(
        &mut self,
        expected_content_type: &ExpectedContentType,
        model:                 &LanguageModelType,
        input_tokens:          &[W::Seed]
    ) -> Result<(),W::Error>
    {
        info!("Beginning full batch workflow execution");

        self.finish_processing_uncompleted_batches(&expected_content_type).await?;

        let requests = self.compute_language_model_requests(model, input_tokens);

        let batches = construct_batches(&requests, Self::REQUESTS_PER_BATCH);

        for (batch_idx, batch_requests) in batches {
            info!("Processing batch #{}", batch_idx);
            self.process_batch_requests(batch_requests,expected_content_type).await?;
        }

        Ok(())
    }
}
