crate::ix!();

#[async_trait]
impl<P, H> Show for Workspace<P, H>
where
    P: AsRef<Path> + Clone + Send + Sync + 'static,
    H: CrateHandleInterface<P> + ShowCrate<Error = CrateError> + Send + Sync + 'static,
{
    type Error = WorkspaceError;

    #[tracing::instrument(level = "trace", skip(self, options))]
    async fn show(&self, options: &ShowFlags) -> Result<String, Self::Error> {
        trace!("Entering ShowWorkspace::show_workspace at path={:?}", self.as_ref());

        // We'll accumulate the final output across all crates
        let mut output = String::new();

        let crates_list = self.crates();
        if crates_list.is_empty() && options.show_items_with_no_data() {
            return Ok("<no-data-for-crate>\n".to_string());
        }

        // For each crate:
        for crate_arc in crates_list {
            let mut guard = crate_arc.lock().await;
            // Show the crate using the ShowCrate trait, but skip merge_crates logic 
            // since we want each crate separate in a workspace scenario.
            // We'll do a local copy of `options` with merge_crates=false forced if needed,
            // because "workspace" subcommand in the old code never merges them.
            let mut local_opts = options.clone();
            // The old CLI code never merges all crates in a workspace, so we forcibly disable it:
            local_opts.merge_crates = false;

            let cci_str = guard.show(&local_opts).await.map_err(WorkspaceError::CrateError)?;
            let crate_name = guard.name();
            let info_line = format!("--- [Crate: {}] ---\n", crate_name);
            info!("{}", info_line.trim());

            if cci_str.trim().is_empty() {
                if options.show_items_with_no_data() {
                    output.push_str(&info_line);
                    output.push_str("<no-data-for-crate>\n\n");
                }
            } else {
                output.push_str(&info_line);
                output.push_str(&cci_str);
                output.push('\n');
            }
        }

        if output.trim().is_empty() && options.show_items_with_no_data() {
            output = "<no-data>\n".to_string();
        }

        Ok(output)
    }
}
