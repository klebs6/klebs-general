// ---------------- [ File: src/language_model_token_expander.rs ]
crate::ix!();

/// The improved LanguageModelTokenExpander, now with no `pub` fields. Instead, we rely on
/// `getset` to provide getters (and optionally setters) and `derive_builder` for
/// constructing robustly. This struct implements `LanguageModelBatchWorkflow` to unify
/// your batch processing logic under a trait-based approach.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct LanguageModelTokenExpander<T: CreateLanguageModelRequestsAtAgentCoordinate> {
    client:                         Arc<OpenAIClientHandle>,
    workspace:                      Arc<BatchWorkspace>,
    model:                          LanguageModelType,
    language_model_request_creator: Arc<T>,
    inputs:                         Vec<<Self as ComputeLanguageModelRequests>::Seed>,
    unseen_inputs:                  Vec<<Self as ComputeLanguageModelRequests>::Seed>,
    language_model_requests:        Vec<LanguageModelBatchAPIRequest>,
}

unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Send for LanguageModelTokenExpander<T> {}
unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Sync for LanguageModelTokenExpander<T> {}

impl<T:CreateLanguageModelRequestsAtAgentCoordinate> LanguageModelTokenExpander<T> {

    pub async fn new(
        product_root:        impl AsRef<Path>,
        model:               LanguageModelType, 
        language_model_request_creator: Arc<T>

    ) -> Result<Self,TokenExpanderError> {

        info!("creating LanguageModelTokenExpander with model: {:#?}", model);

        Ok(Self {
            client:    OpenAIClientHandle::new(),
            workspace: BatchWorkspace::new_in(product_root).await?,
            model,
            language_model_request_creator,
            inputs:                  vec![],
            unseen_inputs:           vec![],
            language_model_requests: vec![],
        })
    }

    delegate!{
        to self.workspace {
            pub fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn batch_expansion_error_log_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn token_expansion_path(&self,token_name: &<Self as ComputeLanguageModelRequests>::Seed) -> PathBuf;
        }
    }

    /// Internal helper from your original code. Identifies newly seen tokens.
    pub fn calculate_unseen_inputs(&mut self, inputs: &[CamelCaseTokenWithComment]) {

        let mut unseen = Vec::new();

        for tok in inputs {

            let target_path = tok.target_path_for_ai_json_expansion(&self.workspace.target_dir());

            if !target_path.exists() {
                if let Some(similar_path) = find_similar_target_path(&self.workspace,&target_path) {
                    warn!(
                        "Skipping token '{}': target path '{}' is similar to existing '{}'.",
                        tok.data(),
                        target_path.display(),
                        similar_path.display()
                    );
                    continue; // Skip this token
                }
                unseen.push(tok.clone());
            }
        }

        self.unseen_inputs = unseen;

        info!("Unseen input tokens calculated:");

        for token in &self.unseen_inputs {
            info!("{}", token);
        }
    }
}

impl<T> GetBatchWorkspace<BatchWorkspaceError> for LanguageModelTokenExpander<T>
where T: CreateLanguageModelRequestsAtAgentCoordinate
{
    fn workspace(&self) -> Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>> {
        self.workspace.clone()
    }
}

impl<T, E> GetLanguageModelClient<E> for LanguageModelTokenExpander<T>
where
    T: CreateLanguageModelRequestsAtAgentCoordinate,
    OpenAIClientHandle: LanguageModelClientInterface<E>,
{
    fn language_model_client(&self) -> Arc<dyn LanguageModelClientInterface<E>> {
        self.client.clone()
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

        let mut triple = BatchFileTriple::new_with_requests(batch_requests, self.workspace().clone())?;

        let execution_result = triple.fresh_execute(&self.client()).await?;

        let workspace = self.workspace();

        process_batch_output_and_errors(&**workspace, &execution_result,&expected_content_type).await?;

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
    type Error = TokenExpanderError;

    fn compute_language_model_requests(
        &mut self,
        model:            &LanguageModelType,
        agent_coordinate: &AgentCoordinate,
        inputs:           &[Self::Seed]

    ) -> Vec<LanguageModelBatchAPIRequest> {

        trace!("Computing GPT requests from newly provided tokens...");
        self.calculate_unseen_inputs(inputs);
        self.language_model_request_creator().create_language_model_requests_at_agent_coordinate(
            model,
            agent_coordinate,
            &self.unseen_inputs()
        )
    }
}
