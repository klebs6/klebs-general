// ---------------- [ File: src/these_will_go_away_somehow.rs ]
crate::ix!();

#[async_trait]
impl<T> FinishProcessingUncompletedBatches for LanguageModelTokenExpander<T>
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    type Error = TokenExpanderError;

    async fn finish_processing_uncompleted_batches(
        &self,
        expected_content_type: &ExpectedContentType
    ) -> Result<(), Self::Error> 
    {
        info!("Finishing uncompleted batches if any remain.");

        let workspace             = self.workspace();
        let language_model_client = self.language_model_client();

        let mut batch_triples = workspace.clone().gather_all_batch_triples().await?;
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
            triple.reconcile_unprocessed(
                    &*language_model_client,
                    expected_content_type,
                    &PROCESS_OUTPUT_FILE_BRIDGE,
                    &PROCESS_ERROR_FILE_BRIDGE,
                ).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T> ProcessBatchRequests for LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    type Error = TokenExpanderError;

    async fn process_batch_requests(
        &self,
        batch_requests:        &[LanguageModelBatchAPIRequest],
        expected_content_type: &ExpectedContentType,
    ) -> Result<(), Self::Error> {

        info!("Processing {} batch request(s)", batch_requests.len());

        let workspace = self.workspace();

        let mut triple = BatchFileTriple::new_with_requests(batch_requests, workspace.clone())?;

        let execution_result = triple.fresh_execute(&self.client()).await?;

        process_batch_output_and_errors(&**workspace, &execution_result, &expected_content_type).await?;

        triple.move_all_to_done().await?;

        Ok(())
    }
}

//-------------------------------------------[everything-below-here]
// we want this to be done by LanguageModelBatchWorkflow derive macro, as well as the //actual
// derivation of LanguageModelBatchWorkflow trait

#[async_trait]
impl<T:CreateLanguageModelRequestsAtAgentCoordinate> 
LanguageModelBatchWorkflow<TokenExpanderError> for LanguageModelTokenExpander<T> {}

unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Send for LanguageModelTokenExpander<T> {}
unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Sync for LanguageModelTokenExpander<T> {}

impl<T> GetBatchWorkspace<BatchWorkspaceError> for LanguageModelTokenExpander<T>
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    fn workspace(&self) -> Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>> {
        self.workspace().clone()
    }
}

impl<T> GetLanguageModelClient<OpenAIClientError> for LanguageModelTokenExpander<T>
where T: CreateLanguageModelRequestsAtAgentCoordinate,
{
    fn language_model_client(&self) -> Arc<dyn LanguageModelClientInterface<OpenAIClientError>> {
        self.client().clone()
    }
}
