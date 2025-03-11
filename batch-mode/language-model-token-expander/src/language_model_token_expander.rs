// ---------------- [ File: src/language_model_token_expander.rs ]
crate::ix!();

/// The improved LanguageModelTokenExpander, now with no `pub` fields. Instead, we rely on
/// `getset` to provide getters (and optionally setters) and `derive_builder` for
/// constructing robustly. This struct implements `LanguageModelBatchWorkflow` to unify
/// your batch processing logic under a trait-based approach.
#[derive(Getters,LanguageModelBatchWorkflow)]
#[getset(get = "pub")]
#[batch_error_type(TokenExpanderError)]
pub struct LanguageModelTokenExpander<T: CreateLanguageModelRequestsAtAgentCoordinate> {

    language_model_request_creator: Arc<T>,
    agent_coordinate:               AgentCoordinate,

    #[batch_client]                   client:                  Arc<dyn LanguageModelClientInterface<TokenExpanderError>>,
    #[batch_workspace]                workspace:               Arc<BatchWorkspace>,
    #[expected_content_type]          expected_content_type:   ExpectedContentType,
    #[model_type]                     language_model_type:     LanguageModelType,
}

impl<T> LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    pub async fn new(
        product_root:                   impl AsRef<Path>,
        language_model_request_creator: Arc<T>,
        agent_coordinate:               AgentCoordinate,
        language_model_type:            LanguageModelType,
        expected_content_type:          ExpectedContentType,

    ) -> Result<Self,TokenExpanderError> {

        info!("creating LanguageModelTokenExpander");

        let client: Arc<dyn LanguageModelClientInterface<TokenExpanderError>> = OpenAIClientHandle::new();

        Ok(Self {
            language_model_request_creator,
            agent_coordinate,
            client,
            workspace:     BatchWorkspace::new_in(product_root).await?,
            expected_content_type,
            language_model_type,
        })
    }
}

impl<T> ComputeSystemMessage for LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    fn system_message() -> String {
        todo!();
    }
}

/// Here we implement the trait that organizes all batch-processing stages.
impl<T> ComputeLanguageModelCoreQuery for LanguageModelTokenExpander<T> 
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    type Seed  = CamelCaseTokenWithComment;

    fn compute_language_model_core_query(
        &self,
        input: &Self::Seed

    ) -> String {

        trace!("Computing query core from seed...");

        todo!();
        /*

        let workspace = self.workspace();

        let unseen = workspace.calculate_unseen_inputs(inputs);

        self.language_model_request_creator().create_language_model_requests_at_agent_coordinate(
            model,
            &self.agent_coordinate,
            &unseen
        )
        */
    }
}
