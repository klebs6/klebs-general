// That Yogin who is freed from attachment and pride, who transcends all pairs of opposites such
// as pleasure and pain, who never gives way to wrath or hate, who never speaks an untruth, who
// though slandered or struck still shows friendship for the slanderer or the striker, who never
// thinks of doing ill to others, who restrains these three, viz. speech, acts and mind, and who
// behaves uniformly towards all creatures, succeeds in approaching Brahman (true self).
// 
// — The Mahabharata, Shanti Parva, Chapter CCXXXVI, 
// ---------------- [ File: src/language_model_token_expander.rs ]
crate::ix!();

/// The improved LanguageModelTokenExpander, now with no `pub` fields. Instead, we rely on
/// `getset` to provide getters (and optionally setters) and `derive_builder` for
/// constructing robustly. This struct implements `LanguageModelBatchWorkflow` to unify
/// your batch processing logic under a trait-based approach.
//#[derive(Getters,LanguageModelBatchWorkflow)]
#[derive(Getters)]
#[getset(get = "pub")]
pub struct LanguageModelTokenExpander<T: CreateLanguageModelRequestsAtAgentCoordinate> {

    language_model_request_creator: Arc<T>,

    //#[batch_client]       
    client:    Arc<OpenAIClientHandle>,

    //#[batch_workspace] 
    workspace: Arc<BatchWorkspace>,
}

impl<T:CreateLanguageModelRequestsAtAgentCoordinate> LanguageModelTokenExpander<T> {

    pub async fn new(
        product_root:                   impl AsRef<Path>,
        language_model_request_creator: Arc<T>

    ) -> Result<Self,TokenExpanderError> {
        info!("creating LanguageModelTokenExpander");
        Ok(Self {
            language_model_request_creator,
            client:        OpenAIClientHandle::new(),
            workspace:     BatchWorkspace::new_in(product_root).await?,
        })
    }
}

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

/// Here we implement the trait that organizes all batch-processing stages.
#[async_trait]
impl<T> ComputeLanguageModelRequests for LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    type Seed  = CamelCaseTokenWithComment;

    fn compute_language_model_requests(
        &self,
        model:            &LanguageModelType,
        agent_coordinate: &AgentCoordinate,
        inputs:           &[Self::Seed]

    ) -> Vec<LanguageModelBatchAPIRequest> {

        trace!("Computing GPT requests from newly provided tokens...");

        let workspace = self.workspace();

        let unseen = workspace.calculate_unseen_inputs(inputs);

        self.language_model_request_creator().create_language_model_requests_at_agent_coordinate(
            model,
            agent_coordinate,
            &unseen
        )
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
        self.workspace.clone()
    }
}

impl<T> GetLanguageModelClient<OpenAIClientError> for LanguageModelTokenExpander<T>
where T: CreateLanguageModelRequestsAtAgentCoordinate,
{
    fn language_model_client(&self) -> Arc<dyn LanguageModelClientInterface<OpenAIClientError>> {
        self.client.clone()
    }
}

