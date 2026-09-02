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

#[tracing::instrument(level = "info", skip(writer, requests), fields(plant = plant, request_count = requests.len()))]
pub async fn execute_ai_readme_writer_requests(
    writer:   &mut AiReadmeWriter,
    requests: &[AiReadmeWriterRequest<PathBuf>],
    plant:    bool
) -> Result<(), AiReadmeWriterError>
{
    if requests.is_empty() {
        info!("execute_ai_readme_writer_requests: no requests provided; nothing to do.");
        return Ok(());
    }

    let unseen = writer
        .batch_workspace()
        .calculate_unseen_inputs(requests, &ExpectedContentType::Json);

    info!(
        "execute_ai_readme_writer_requests: scanned batch workspace; total_requests={}, unseen_requests={}, plant={}",
        requests.len(),
        unseen.len(),
        plant
    );

    if !unseen.is_empty() {
        debug!(
            "execute_ai_readme_writer_requests: unseen crate targets = {:?}",
            unseen
                .iter()
                .map(|r| r.crate_name().clone())
                .collect::<Vec<String>>()
        );
    }

    if plant {
        if unseen.is_empty() {
            info!("execute_ai_readme_writer_requests: no unseen requests; skipping plant_seed_and_wait.");
        } else {
            info!(
                "execute_ai_readme_writer_requests: planting {} unseen request(s) into batch workflow.",
                unseen.len()
            );
            writer.plant_seed_and_wait(&unseen).await?;
            info!("execute_ai_readme_writer_requests: plant_seed_and_wait completed.");
        }
    } else {
        info!("execute_ai_readme_writer_requests: plant=false; will only apply results already present in the batch workspace.");
    }

    let results = writer.gather_results(requests).await?;

    debug!(
        "execute_ai_readme_writer_requests: gathered {} result(s) for {} request(s).",
        results.len(),
        requests.len()
    );

    let produced_crates: std::collections::HashSet<String> = results
        .iter()
        .map(|(seed, _output)| seed.crate_name().clone())
        .collect();

    let missing_crates: Vec<String> = requests
        .iter()
        .filter(|req| !produced_crates.contains(req.crate_name()))
        .map(|req| req.crate_name().clone())
        .collect();

    if !missing_crates.is_empty() {
        if plant {
            error!(
                "execute_ai_readme_writer_requests: missing AI outputs after planting. missing_count={}, missing_crates={:?}",
                missing_crates.len(),
                missing_crates
            );
        } else {
            warn!(
                "execute_ai_readme_writer_requests: some requested crates have no AI output available in the batch workspace (plant disabled). missing_count={}, missing_crates={:?}",
                missing_crates.len(),
                missing_crates
            );
        }
    }

    for (request, response) in results {
        let crate_name = request.crate_name().to_string();
        let readme_len = response.full_readme_markdown().len();
        let kw_len = response.package_keywords().len();
        let cat_len = response.package_categories().len();

        info!(
            "execute_ai_readme_writer_requests: applying AI output for crate='{}' (readme_chars={}, keywords={}, categories={})",
            crate_name,
            readme_len,
            kw_len,
            cat_len
        );

        let handle = request.crate_handle();
        let guard  = handle.lock().await;

        guard.update_readme_md(response.full_readme_markdown()).await?;
        guard.update_cargo_toml(
            response.package_description(),
            response.package_keywords(),
            response.package_categories(),
        ).await?;

        info!(
            "execute_ai_readme_writer_requests: successfully applied AI output for crate='{}'",
            crate_name
        );
    }

    if plant && !missing_crates.is_empty() {
        return Err(AiReadmeWriterError::ReadmeWriteError(
            ReadmeWriteError::AiReadmeWriterError,
        ));
    }

    if produced_crates.is_empty() {
        warn!(
            "execute_ai_readme_writer_requests: no AI outputs were applied. total_requests={}, plant={}",
            requests.len(),
            plant
        );
    }

    info!("execute_ai_readme_writer_requests: completed.");
    Ok(())
}
