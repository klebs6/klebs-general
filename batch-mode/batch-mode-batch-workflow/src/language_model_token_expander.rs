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
    agent_coordinate:               AgentCoordinate,

    //#[batch_client]       
    client:    Arc<OpenAIClientHandle>,

    //#[batch_workspace] 
    workspace: Arc<BatchWorkspace>,

    //#[custom_process_batch_output_fn]
    //#[custom_process_batch_error_fn]
    //#[expected_content_type]
    //#[model_type]
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
