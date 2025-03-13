// ---------------- [ File: workspacer-readme-writer/src/update_readmes.rs ]
crate::ix!();

#[async_trait]
pub trait UpdateReadmeFiles {
    type Error;

    /// Orchestrates the steps to generate queries, call the AI, and update README(s).
    async fn update_readme_files(x: Arc<Self>) -> Result<(), Self::Error>;
}

#[async_trait]
impl UpdateReadmeFiles for CrateHandle {

    type Error = ReadmeWriterExecutionError;

    async fn update_readme_files(x: Arc<Self>) -> Result<(), Self::Error> {

        trace!("Entering CrateHandle::update_readme");

        let requests = vec![AiReadmeWriterRequest::async_try_from::<PathBuf,CrateHandle>(x).await?];

        execute_ai_readme_writer_requests(&requests).await?;

        info!("Exiting CrateHandle::update_readme with success");

        Ok(())
    }
}

#[async_trait]
impl<P,H> UpdateReadmeFiles for Workspace<P,H>
where
    for<'x> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'x,
    H: CrateHandleInterface<P> 
      + ConsolidateCrateInterface
      + Sync + Send + 'static,
{
    type Error = ReadmeWriterExecutionError;

    async fn update_readme_files(x: Arc<Self>) -> Result<(), Self::Error> {

        //TODO: we want to create a batch containing a request for each crate
        // instead of a batch for each crate.
        trace!("Entering Workspace update_readme");

        let mut requests = Vec::new();

        for item in x.crates().iter() {
            let request = AiReadmeWriterRequest::async_try_from::<P,H>(item.clone()).await?;
            requests.push(request);
        }

        execute_ai_readme_writer_requests(&requests).await?;

        info!("Exiting Workspace update_readme with success");

        Ok(())
    }
}
