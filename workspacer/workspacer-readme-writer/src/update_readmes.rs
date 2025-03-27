// ---------------- [ File: workspacer-readme-writer/src/update_readmes.rs ]
crate::ix!();

#[async_trait]
pub trait UpdateReadmeFiles {
    type Error;

    /// Orchestrates the steps to generate queries, call the AI, and update README(s).
    async fn update_readme_files(handle: Arc<AsyncMutex<Self>>, plant: bool) -> Result<(), Self::Error>;
}

#[async_trait]
impl UpdateReadmeFiles for CrateHandle {

    type Error = AiReadmeWriterError;

    async fn update_readme_files(crate_handle: Arc<AsyncMutex<Self>>, plant: bool) -> Result<(), Self::Error> {

        trace!("Entering CrateHandle::update_readme");

        let mut writer = AiReadmeWriter::default().await?;

        let requests = vec![
            AiReadmeWriterRequest::<PathBuf>::async_try_from::<CrateHandle>(crate_handle).await?
        ];

        execute_ai_readme_writer_requests(&mut writer,&requests,plant).await?;

        info!("Exiting CrateHandle::update_readme with success");

        Ok(())
    }
}

#[async_trait]
impl<H> UpdateReadmeFiles for Workspace<PathBuf,H>
where H: ReadmeWritingCrateHandle<PathBuf>,
{
    type Error = AiReadmeWriterError;

    async fn update_readme_files(workspace_handle: Arc<AsyncMutex<Self>>, plant: bool) -> Result<(), Self::Error> {

        //TODO: we want to create a batch containing a request for each crate
        // instead of a batch for each crate.
        trace!("Entering Workspace update_readme");

        let mut writer = AiReadmeWriter::default().await?;

        let requests = {

            let guard = workspace_handle.lock().await;

            let mut requests = Vec::new();

            for item in guard.crates().iter() {
                let request = AiReadmeWriterRequest::<PathBuf>::async_try_from::<H>(item.clone()).await?;
                requests.push(request);
            }

            requests
        };

        execute_ai_readme_writer_requests(&mut writer,&requests,plant).await?;

        info!("Exiting Workspace update_readme with success");

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
