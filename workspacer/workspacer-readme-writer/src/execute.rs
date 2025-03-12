crate::ix!();

pub async fn execute_ai_readme_writer_requests(requests: &[AiReadmeWriterRequest])
    -> Result<(),ReadmeWriterExecutionError>
{
    let readme_dir = WorkspacerDir::local().ensure_subdir_exists("readme-writer-workspace")?;

    let writer = AiReadmeWriter::new(
        readme_dir, 
        LanguageModelType::O1
    ).await?;

    writer.plant_seed_and_wait(requests).await?;

    // TODO: may need to get this from the workspace somehow
    let ai_generated_output = writer.gather_plants().await?;

    for item in ai_generated_output {
        let full_readme_markdown = item.full_readme_markdown();
        let package_description  = item.package_description();
        let package_keywords     = item.package_keywords();
        let package_categories   = item.package_categories();
        todo!();
    }

    todo!();
}
