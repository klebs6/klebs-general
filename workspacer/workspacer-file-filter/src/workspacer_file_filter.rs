// ---------------- [ File: workspacer-file-filter/src/workspacer_file_filter.rs ]
crate::ix!();

#[derive(Builder,Getters,LanguageModelBatchWorkflow)]
#[builder(setter(into))]
#[getset(get = "pub")]
#[batch_error_type(LanguageModelBatchWorkflowError)]
#[batch_json_output_format(AiFileFilterDesiredOutput)]
pub struct AiFileFilter
{
    #[batch_client] 
    language_model_client: LanguageModelClientArc,

    #[model_type] 
    language_model_type:   LanguageModelType,

    #[batch_workspace] 
    batch_workspace:       Arc<BatchWorkspace>,
}

impl AiFileFilter
{
    pub async fn default() -> Result<Self,AiFileFilterError> {
        let readme_dir = WorkspacerDir::local().ensure_subdir_exists("file-filter-workspace")?;
        Ok(AiFileFilter::new(&readme_dir, LanguageModelType::Gpt4_5Preview).await?)
    }

    pub async fn with_model(model_type: &LanguageModelType) -> Result<Self,AiFileFilterError> {
        let readme_dir = WorkspacerDir::local().ensure_subdir_exists("file-filter-workspace")?;
        Ok(AiFileFilter::new(&readme_dir, *model_type).await?)
    }

    pub async fn new(
        workspace_root:      impl AsRef<Path>,
        language_model_type: LanguageModelType,

    ) -> Result<Self,AiFileFilterError> {

        let language_model_client: LanguageModelClientArc = OpenAIClientHandle::new();

        Ok(Self {
            language_model_client,
            language_model_type,
            batch_workspace: BatchWorkspace::new_in(workspace_root).await?,
        })
    }
}

impl ComputeSystemMessage for AiFileFilter
{
    fn system_message() -> String {
        formatdoc!{r#"
            You are assisting us with precise transformations of text file content.

            For each file provided, you'll receive:

            1) Clear instructions detailing how to filter, modify, or rewrite the provided content.

            2) The original text content itself.

            Please produce the modified text exactly as instructed, without additional commentary or explanation.
        "#}
    }
}

impl ComputeLanguageModelCoreQuery for AiFileFilter
{
    type Seed  = AiFileFilterRequest<PathBuf>;

    fn compute_language_model_core_query(
        &self,
        input:           &Self::Seed
    ) -> String {

        let orig_text    = input.original_text();
        let instructions = input.user_instructions();

        formatdoc! {
            r#"
            {orig_text}
            {instructions}
            "#
        }
    }
}
