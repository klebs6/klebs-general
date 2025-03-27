// ---------------- [ File: workspacer-readme-writer/src/readme_writer.rs ]
crate::ix!();

#[derive(Builder,Getters,LanguageModelBatchWorkflow)]
#[builder(setter(into))]
#[getset(get = "pub")]
#[batch_error_type(LanguageModelBatchWorkflowError)]
#[batch_json_output_format(AiReadmeWriterDesiredOutput)]
pub struct AiReadmeWriter
{
    #[batch_client] 
    language_model_client: LanguageModelClientArc,

    #[model_type] 
    language_model_type:   LanguageModelType,

    #[batch_workspace] 
    batch_workspace:       Arc<BatchWorkspace>,
}

impl AiReadmeWriter
{
    pub async fn default() -> Result<Self,AiReadmeWriterError> {
        let readme_dir = WorkspacerDir::local().ensure_subdir_exists("readme-writer-workspace")?;
        Ok(AiReadmeWriter::new(&readme_dir, LanguageModelType::Gpt4o).await?)
    }

    pub async fn new(
        workspace_root:      impl AsRef<Path>,
        language_model_type: LanguageModelType,

    ) -> Result<Self,AiReadmeWriterError> {

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
        formatdoc!{r#"
            We would like you to write a README.md for a rust crate with maximally helpful content.
            Additionally, we would like you to generate several fields from the Cargo.toml package section for this crate.
            We will show you the strict json output format we would like you to provide.
            Please tailor your response for an apex consumer of maximal intelligence.
            A reader of this README.md should have a good sense of what the crate does and how to use it after reading what you write.
        "#}
    }
}

impl ComputeLanguageModelCoreQuery for AiReadmeWriter
{
    type Seed  = AiReadmeWriterRequest<PathBuf>;

    fn compute_language_model_core_query(
        &self,
        input:           &Self::Seed
    ) -> String {

        //NOTE: IMPORTANT: this is different than `input.name()`, which will give us not-the-right-result
        let crate_name = input.crate_name();

        let version    = input.version();
        let interface  = input.consolidated_crate_interface();

        let mut query = formatdoc!{r#"
            We have a rust crate named '{crate_name}' {version}. 

            Here is the interface:
            {interface}
        "#};

        if let Some(authors) = input.maybe_cargo_toml_package_authors() {
            query.push_str(
                &formatdoc!{r#"
                    This crate has the following authors:
                    {authors:#?}
                "#}
            );
        }

        if let Some(edition) = input.maybe_cargo_toml_rust_edition() {
            query.push_str(
                &formatdoc!{r#"
                    This crate has the following rust edition:
                    {edition}
                "#}
            );

        }

        if let Some(license) = input.maybe_cargo_toml_license() {
            query.push_str(
                &formatdoc!{r#"
                    This crate has the following license:
                    {license}
                "#}
            );
        }

        if let Some(repository) = input.maybe_cargo_toml_crate_repository_location() {
            query.push_str(
                &formatdoc!{r#"
                    This crate is held in the following repository:
                    {repository}
                "#}
            );
        }

        query
    }
}
