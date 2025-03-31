// ---------------- [ File: workspacer-readme-writer/src/update_readmes.rs ]
crate::ix!();

#[async_trait]
pub trait UpdateReadmeFiles {
    type Error;

    /// Orchestrates the steps to generate queries, call the AI, and update README(s).
    /// The `force` parameter means: if false, skip crates that already have a README file.
    async fn update_readme_files(
        handle: Arc<AsyncMutex<Self>>,
        plant: bool,
        force: bool
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl UpdateReadmeFiles for CrateHandle {
    type Error = AiReadmeWriterError;

    async fn update_readme_files(
        crate_handle: Arc<AsyncMutex<Self>>,
        plant: bool,
        force: bool
    ) -> Result<(), Self::Error> 
    {
        trace!("Entering CrateHandle::update_readme_files(...) with plant={plant}, force={force}");

        {
            // Quickly check if a README.md already exists:
            let guard = crate_handle.lock().await;
            let maybe_readme = guard.readme_path().await.map_err(AiReadmeWriterError::CrateError)?;

            if maybe_readme.is_some() && !force {
                // We skip entirely:
                info!(
                    "Skipping crate at {:?} because README.md already exists and --force was not specified.",
                    guard.as_ref()
                );
                return Ok(());  
            }
        }

        // If we *don't* skip, do the actual AI steps as before:
        let mut writer = AiReadmeWriter::default().await?;
        let requests = vec![
            AiReadmeWriterRequest::<PathBuf>::async_try_from::<CrateHandle>(crate_handle).await?
        ];

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
        force: bool
    ) -> Result<(), Self::Error> 
    {
        trace!("Entering Workspace update_readme_files(...) with plant={plant}, force={force}");

        let mut writer = AiReadmeWriter::default().await?;

        // We'll build a `requests` Vec, but skip crates that already have a readme if `!force`.
        let requests = {
            let guard = workspace_arc.lock().await;
            let mut reqs = Vec::new();
            for item_arc in guard.crates() {
                // Check the item’s readme:
                let item_guard = item_arc.lock().await;
                let maybe_readme = item_guard
                    .readme_path()
                    .await
                    .map_err(AiReadmeWriterError::CrateError)?;

                if maybe_readme.is_some() && !force {
                    info!("Skipping crate at {:?} due to existing README.md (no --force)", item_guard.as_ref());
                    continue;
                }

                // If no readme or we forced => we create a request
                let request = AiReadmeWriterRequest::<PathBuf>::async_try_from::<H>(item_arc.clone()).await?;
                reqs.push(request);
            }
            reqs
        };

        // If `requests` is empty, we simply do nothing (though we might want to log a message).
        if requests.is_empty() {
            info!("No crates need README generation (either they have READMEs or no crates at all).");
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
