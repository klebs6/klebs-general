// ---------------- [ File: src/language_model_token_expander.rs ]
crate::ix!();

/// The improved LanguageModelTokenExpander, now with no `pub` fields. Instead, we rely on
/// `getset` to provide getters (and optionally setters) and `derive_builder` for
/// constructing robustly. This struct implements `LanguageModelBatchWorkflow` to unify
/// your batch processing logic under a trait-based approach.
#[derive(Getters,LanguageModelBatchWorkflow)]
#[getset(get = "pub")]
pub struct LanguageModelTokenExpander<T: CreateLanguageModelRequestsAtAgentCoordinate> {

    language_model_request_creator: Arc<T>,
    agent_coordinate:               AgentCoordinate,

    #[batch_client]       
    client:    Arc<OpenAIClientHandle>,

    #[batch_workspace] 
    workspace: Arc<BatchWorkspace>,

    #[custom_process_batch_output_fn]
    process_batch_output_fn: ProcessBatchOutputFn,

    #[custom_process_batch_error_fn]
    process_batch_error_fn: ProcessBatchErrorFn,

    #[expected_content_type]
    expected_content_type: ExpectedContentType,

    #[model_type]
    language_model_type: LanguageModelType,
}

impl<T> LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    pub async fn new(
        product_root:                   impl AsRef<Path>,
        language_model_request_creator: Arc<T>,
        agent_coordinate:               AgentCoordinate,

    ) -> Result<Self,TokenExpanderError> {
        info!("creating LanguageModelTokenExpander");
        Ok(Self {
            language_model_request_creator,
            agent_coordinate,
            client:        OpenAIClientHandle::new(),
            workspace:     BatchWorkspace::new_in(product_root).await?,
        })
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
        inputs:           &[Self::Seed]

    ) -> Vec<LanguageModelBatchAPIRequest> {

        trace!("Computing GPT requests from newly provided tokens...");

        let workspace = self.workspace();

        let unseen = workspace.calculate_unseen_inputs(inputs);

        self.language_model_request_creator().create_language_model_requests_at_agent_coordinate(
            model,
            &self.agent_coordinate,
            &unseen
        )
    }
}
