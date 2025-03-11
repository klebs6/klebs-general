// ---------------- [ File: workspacer-readme-writer/src/readme_writer.rs ]
crate::ix!();

#[derive(Getters,LanguageModelBatchWorkflow)]
#[getset(get = "pub")]
#[batch_error_type(LanguageModelBatchWorkflowError)]
#[batch_json_output_format(AiReadmeWriterDesiredOutput)]
pub struct AiReadmeWriter {

    #[batch_client] 
    language_model_client: LanguageModelClientArc,

    #[model_type] 
    language_model_type:   LanguageModelType,

    #[batch_workspace] 
    batch_workspace:       Arc<BatchWorkspace>,
}

impl AiReadmeWriter 
{
    pub async fn new(
        workspace_root:      impl AsRef<Path>,
        language_model_type: LanguageModelType,

    ) -> Result<Self,LanguageModelBatchWorkflowError> {

        let language_model_client: LanguageModelClientArc = OpenAIClientHandle::new();

        Ok(Self {
            language_model_client,
            language_model_type,
            batch_workspace: BatchWorkspace::new_in(workspace_root).await?,
        })
    }
}

impl ComputeSystemMessage for AiReadmeWriter 
{
    fn system_message() -> String {
        todo!();
    }
}

impl ComputeLanguageModelCoreQuery for AiReadmeWriter 
{
    type Seed  = AiReadmeWriterRequest;

    fn compute_language_model_core_query(
        &self,
        input:           &Self::Seed
    ) -> String {

        let query: String = todo!("depending on what we want to do, we can use the seed as well as the struct members to compute this here");
        todo!();
    }
}
