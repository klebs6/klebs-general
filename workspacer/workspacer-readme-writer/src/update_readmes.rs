// ---------------- [ File: workspacer-readme-writer/src/update_readmes.rs ]
crate::ix!();

#[async_trait]
pub trait UpdateReadmeFiles {
    type Error;

    /// Orchestrates the steps to generate queries, call the AI, and update README(s).
    async fn update_readme_files(x: Arc<AsyncMutex<Self>>) -> Result<(), Self::Error>;
}

#[async_trait]
impl UpdateReadmeFiles for CrateHandle {

    type Error = ReadmeWriterExecutionError;

    async fn update_readme_files(x: Arc<AsyncMutex<Self>>) -> Result<(), Self::Error> {

        trace!("Entering CrateHandle::update_readme");

        let requests = vec![AiReadmeWriterRequest::<PathBuf>::async_try_from::<CrateHandle>(x).await?];

        execute_ai_readme_writer_requests(&requests).await?;

        info!("Exiting CrateHandle::update_readme with success");

        Ok(())
    }
}

#[async_trait]
impl<H> UpdateReadmeFiles for Workspace<PathBuf,H>
where H: ReadmeWritingCrateHandle<PathBuf>,
{
    type Error = ReadmeWriterExecutionError;

    async fn update_readme_files(x: Arc<AsyncMutex<Self>>) -> Result<(), Self::Error> {

        //TODO: we want to create a batch containing a request for each crate
        // instead of a batch for each crate.
        trace!("Entering Workspace update_readme");

        let guard = x.lock().await;

        let mut requests = Vec::new();

        for item in guard.crates().iter() {
            let request = AiReadmeWriterRequest::<PathBuf>::async_try_from::<H>(item.clone()).await?;
            requests.push(request);
        }

        execute_ai_readme_writer_requests(&requests).await?;

        info!("Exiting Workspace update_readme with success");

        Ok(())
    }
}
