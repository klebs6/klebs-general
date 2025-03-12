// ---------------- [ File: workspacer-readme-writer/src/update_workspace_readmes_example.rs ]
crate::ix!();

#[async_trait]
impl<P,H> UpdateReadmes for Workspace<P,H>
where
    for<'x> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'x,
    H: CrateHandleInterface<P> 
      + GenerateReadmeQueries<Error=CrateError>
      + PerformReadmeUpdates<Error=CrateError>
      + ConsolidateCrateInterface
      + Sync + Send + 'static,
{
    type Error = WorkspaceError;

    async fn update_readmes(&self) -> Result<(), Self::Error> {

        trace!("Entering update_workspace_readmes_example");

        for (idx, crate_handle) in self.crates().iter().enumerate() {
            info!("Processing crate #{} in workspace: {:?}", idx, crate_handle.as_ref());
            let consolidation_opts = ConsolidationOptions::new().with_docs().with_fn_bodies();

            let interface = crate_handle.consolidate_crate_interface(&consolidation_opts)
                .await
                .map_err(WorkspaceError::CrateError)?;
            let queries = crate_handle.generate_readme_queries(&interface)
                .await
                .map_err(WorkspaceError::CrateError)?;

            todo!();

            /*
            let responses = send_readme_queries_to_ai(&queries)
                .await
                .map_err(WorkspaceError::CrateError)?;

            if let Some(first_response) = responses.first() {
                crate_handle.write_updated_readme(first_response)
                    .await
                    .map_err(WorkspaceError::CrateError)?;
                } else {
                    warn!("No AI response for crate #{} => skipping README update", idx);
            }
            */
        }

        // Optionally, also update a top-level workspace README:
        debug!("Now generating the top-level workspace readme query");

        let workspace_interface = ConsolidatedCrateInterface::new();
        let workspace_queries   = self.generate_readme_queries(&workspace_interface).await?;

        todo!();

        /*
        let workspace_responses = send_readme_queries_to_ai(&workspace_queries).await
            .map_err(WorkspaceError::CrateError)?;

        if let Some(ws_resp) = workspace_responses.first() {
            self.write_updated_readme(ws_resp).await?;
        } else {
            warn!("No AI response for workspace-level README => skipping top-level README update");
        }
        */

        info!("Exiting update_workspace_readmes_example with success");
        Ok(())
    }
}
