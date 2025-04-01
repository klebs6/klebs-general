// ---------------- [ File: workspacer-readme-writer/src/update_readmes.rs ]
crate::ix!();

#[async_trait]
pub trait UpdateReadmeFiles {
    type Error;

    /// Orchestrates the steps to generate queries, call the AI, and update README(s).
    /// The `force` parameter means: if false, skip crates/workspaces that already have a README.
    /// The `config` carries user preferences (docs/fn-bodies/test-items/etc.) for readme generation.
    async fn update_readme_files(
        handle: Arc<AsyncMutex<Self>>,
        plant: bool,
        force: bool,
        config: &ReadmeWriterConfig
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl UpdateReadmeFiles for CrateHandle {
    type Error = AiReadmeWriterError;

    async fn update_readme_files(
        crate_handle: Arc<AsyncMutex<Self>>,
        plant: bool,
        force: bool,
        config: &ReadmeWriterConfig
    ) -> Result<(), Self::Error>
    {
        trace!("Entering CrateHandle::update_readme_files(...) with plant={}, force={}", plant, force);

        {
            // Quickly check if a README.md already exists:
            let guard = crate_handle.lock().await;
            let maybe_readme = guard.readme_path().await.map_err(AiReadmeWriterError::CrateError)?;
            if maybe_readme.is_some() && !force {
                info!(
                    "Skipping crate at {:?} because README.md already exists and --force was not specified.",
                    guard.as_ref()
                );
                return Ok(());
            }
        }

        // If not skipped, do the AI steps:
        let mut writer = AiReadmeWriter::default().await?;
        let request = AiReadmeWriterRequest::<PathBuf>::async_try_from::<CrateHandle>(
            crate_handle,
            config
        ).await?;

        let requests = vec![request];
        execute_ai_readme_writer_requests(&mut writer, &requests, plant).await?;

        info!("Exiting CrateHandle::update_readme_files(...) with success");
        Ok(())
    }
}

#[async_trait]
impl<H> UpdateReadmeFiles for Workspace<PathBuf,H>
where
    H: ReadmeWritingCrateHandle<PathBuf>,
{
    type Error = AiReadmeWriterError;

    async fn update_readme_files(
        workspace_arc: Arc<AsyncMutex<Self>>,
        plant: bool,
        force: bool,
        config: &ReadmeWriterConfig
    ) -> Result<(), Self::Error>
    {
        trace!("Entering Workspace update_readme_files(...) with plant={}, force={}", plant, force);

        let mut writer = AiReadmeWriter::default().await?;

        let requests = {
            let guard = workspace_arc.lock().await;
            let mut reqs = Vec::new();

            for item_arc in guard.crates() {
                // 1) Lock once to check if we should skip
                let skip_this = {
                    let item_guard = item_arc.lock().await;
                    let maybe_readme = item_guard.readme_path().await?;
                    // Evaluate skip logic
                    if maybe_readme.is_some() && !force {
                        true
                    } else {
                        false
                    }
                    // item_guard is dropped here
                };

                // 2) If skipping, continue
                if skip_this {
                    info!("Skipping crate because README already exists, no --force");
                    continue;
                }

                // 3) Now create the request, which will lock item_arc internally without deadlock
                let request = AiReadmeWriterRequest::async_try_from::<H>(
                    item_arc.clone(),
                    config
                ).await?;
                reqs.push(request);
            }
            reqs
        };

        if requests.is_empty() {
            info!("No crates need README generation in this workspace.");
            return Ok(());
        }

        execute_ai_readme_writer_requests(&mut writer, &requests, plant).await?;

        info!("Exiting Workspace update_readme_files(...) with success.");
        Ok(())
    }
}

pub async fn execute_ai_readme_writer_requests(
    writer:   &mut AiReadmeWriter,
    requests: &[AiReadmeWriterRequest<PathBuf>],
    plant:    bool,
) -> Result<(), AiReadmeWriterError>
{
    let unseen = writer.batch_workspace().calculate_unseen_inputs(&requests,&ExpectedContentType::Json);

    info!("Gathering AI expansions from the workspace. unseen inputs={:#?}", unseen);

    if plant {
        writer.plant_seed_and_wait(&unseen).await?;
    }

    let results = writer.gather_results(&unseen).await?;

    debug!("Gathered results={:#?}",results);
    for (request, response) in results {

        // Because request.crate_handle() => Arc<dyn ReadmeWritingCrateHandle<P>>
        // that has .update_readme_md + .update_cargo_toml
        let handle = request.crate_handle();
        let guard  = handle.lock().await;

        guard.update_readme_md(response.full_readme_markdown()).await?;
        guard.update_cargo_toml(
            response.package_description(),
            response.package_keywords(),
            response.package_categories(),
        ).await?;
    }

    info!("Successfully completed execute_ai_readme_writer_requests.");
    Ok(())
}
