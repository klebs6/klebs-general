// ---------------- [ File: workspacer-readme-writer/src/execute.rs ]
crate::ix!();

pub async fn execute_ai_readme_writer_requests(
    requests: &[AiReadmeWriterRequest<PathBuf>],
) -> Result<(), ReadmeWriterExecutionError>
{
    use tracing::{trace, info};

    trace!("Starting execute_ai_readme_writer_requests with {} request(s).", requests.len());

    let readme_dir = WorkspacerDir::local().ensure_subdir_exists("readme-writer-workspace")?;
    let mut writer = AiReadmeWriter::new(&readme_dir, LanguageModelType::O1).await?;

    info!("Beginning the AI expansions");
    writer.plant_seed_and_wait(requests).await?;

    info!("Gathering AI expansions from the workspace.");
    let results = writer.gather_results(requests).await?;
    for (request, response) in results {
        // Because request.crate_handle() => Arc<dyn ReadmeWritingCrateHandle<P>>
        // that has .update_readme_md + .update_cargo_toml
        let handle = request.crate_handle();
        handle.update_readme_md(response.full_readme_markdown()).await?;
        handle.update_cargo_toml(
            response.package_description(),
            response.package_keywords(),
            response.package_categories(),
        ).await?;
    }

    info!("Successfully completed execute_ai_readme_writer_requests.");
    Ok(())
}
