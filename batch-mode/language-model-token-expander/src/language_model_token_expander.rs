// ---------------- [ File: language-model-token-expander/src/language_model_token_expander.rs ]
crate::ix!();

#[derive(Debug,Getters,LanguageModelBatchWorkflow)]
#[getset(get = "pub")]
#[batch_error_type(TokenExpanderError)]
#[batch_json_output_format(E)]
pub struct LanguageModelTokenExpander<E> 
where E: ExpandedToken
       + DeserializeOwned
       + Named
       + AiJsonTemplate
       + GetTargetPathForAIExpansion
       + LoadFromFile<Error = SaveLoadError> + 'static,
{

    language_model_request_creator: Arc<<E as ExpandedToken>::Expander>,
    agent_coordinate:               AgentCoordinate,

    #[batch_client]          client:                  Arc<dyn LanguageModelClientInterface<TokenExpanderError>>,
    #[batch_workspace]       batch_workspace:         Arc<BatchWorkspace>,
    #[expected_content_type] expected_content_type:   ExpectedContentType,
    #[model_type]            language_model_type:     LanguageModelType,
}

impl<E> LanguageModelTokenExpander<E> 
where E: ExpandedToken
       + DeserializeOwned
       + Named
       + AiJsonTemplate
       + GetTargetPathForAIExpansion
       + LoadFromFile<Error = SaveLoadError> + 'static,

{
    pub async fn new(
        product_root:                   impl AsRef<Path>,
        language_model_request_creator: Arc<<E as ExpandedToken>::Expander>,
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
            batch_workspace:     BatchWorkspace::new_in(product_root).await?,
            expected_content_type,
            language_model_type,
        })
    }
}

impl<E> ComputeSystemMessage for LanguageModelTokenExpander<E> 
where E: ExpandedToken
       + DeserializeOwned
       + Named
       + AiJsonTemplate
       + GetTargetPathForAIExpansion
       + LoadFromFile<Error = SaveLoadError> + 'static,

{
    fn system_message() -> String {
        //TODO: can make this better
        formatdoc!{
            "We are performing a token expansion."
        }
    }
}

/// Here we implement the trait that organizes all batch-processing stages.
impl<E> ComputeLanguageModelCoreQuery for LanguageModelTokenExpander<E> 
where E: ExpandedToken
       + DeserializeOwned
       + Named
       + AiJsonTemplate
       + GetTargetPathForAIExpansion
       + LoadFromFile<Error = SaveLoadError> + 'static,
{
    type Seed  = TokenPackagedForExpansion;

    fn compute_language_model_core_query(
        &self,
        input: &Self::Seed

    ) -> String {

        trace!("Computing query core from seed...");

        let coord   = self.agent_coordinate();
        let model   = self.language_model_type();
        let creator = self.language_model_request_creator();

        creator.create_language_model_query_at_agent_coordinate(
            &model,
            &coord,
            input
        )
    }
}
