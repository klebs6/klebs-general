crate::ix!();

error_tree!{
    pub enum AiTomlWriterError {
        BatchWorkspaceError(BatchWorkspaceError),
        BatchReconciliationError(BatchReconciliationError),
        BatchDownloadError(BatchDownloadError),
        BatchErrorProcessingError(BatchErrorProcessingError),
        BatchMetadataError(BatchMetadataError),
        BatchOutputProcessingError(BatchOutputProcessingError),
        BatchValidationError(BatchValidationError),
        FileMoveError(FileMoveError),
        OpenAIClientError(OpenAIClientError),
        BatchInputCreationError(BatchInputCreationError),
        BatchProcessingError(BatchProcessingError),
        JsonParseError(JsonParseError),
        IOError(std::io::Error),
    }
}

#[derive(Getters,LanguageModelBatchWorkflow)]
#[getset(get = "pub")]
#[batch_error_type(AiTomlWriterError)]
pub struct AiTomlWriter {
    #[batch_client]          client:                  Arc<dyn LanguageModelClientInterface<AiTomlWriterError>>,
    #[batch_workspace]       workspace:               Arc<BatchWorkspace>,
    #[expected_content_type] expected_content_type:   ExpectedContentType,
    #[model_type]            language_model_type:     LanguageModelType,
}

impl AiTomlWriter 
{
    pub async fn new(
        workspace_root:                 impl AsRef<Path>,
        language_model_type:            LanguageModelType,
        expected_content_type:          ExpectedContentType,

    ) -> Result<Self,AiTomlWriterError> {

        info!("creating AiTomlWriter");

        let client: Arc<dyn LanguageModelClientInterface<AiTomlWriterError>> = OpenAIClientHandle::new();

        Ok(Self {
            client,
            workspace:             BatchWorkspace::new_in(workspace_root).await?,
            expected_content_type: ExpectedContentType::Json,
            language_model_type:   LanguageModelType::O1,
        })
    }
}

impl ComputeLanguageModelRequests for AiTomlWriter 
{
    type Seed  = AiTomlWriterRequest;

    fn compute_language_model_requests(
        &self,
        model:            &LanguageModelType,
        inputs:           &[Self::Seed]

    ) -> Vec<LanguageModelBatchAPIRequest> {

        trace!("Computing GPT requests from newly provided tokens...");

        let workspace = self.workspace();

        let unseen = workspace.calculate_unseen_inputs(inputs,&self.expected_content_type);

        let system_message: String = todo!();;

        let queries: Vec<String> = todo!();

        LanguageModelBatchAPIRequest::requests_from_query_strings(&system_message,*model,&queries)
    }
}

#[derive(Debug,Clone)]
pub struct AiTomlWriterRequest {
    crate_name:                   String,
    consolidated_crate_interface: ConsolidatedCrateInterface,
    package_authors:              Vec<String>,
    rust_edition:                 String,
    license:                      String,
    repository:                   String,
    possible_categories:          Vec<(String,String)>,
}

impl std::fmt::Display for AiTomlWriterRequest {

    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(x,"ai-toml-writer-request-for-{}",&self.crate_name)
    }
}

impl Named for AiTomlWriterRequest {

    fn name(&self) -> Cow<'_,str> {
        Cow::Owned(format!("{}-ai-toml-writer-request",&self.crate_name))
    }
}

impl GetTargetPathForAIExpansion for AiTomlWriterRequest {

    fn target_path_for_ai_json_expansion(
        &self, 
        target_dir:            impl AsRef<Path>,
        expected_content_type: &ExpectedContentType,

    ) -> PathBuf {
        todo!();
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AiTomlWriterDesiredOutput {
    package_description: String,
    package_keywords:    Vec<String>,
    package_categories:  Vec<String>,
}
