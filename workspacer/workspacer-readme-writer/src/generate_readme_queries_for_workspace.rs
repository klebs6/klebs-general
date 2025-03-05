// ---------------- [ File: workspacer-readme-writer/src/generate_readme_queries_for_workspace.rs ]
crate::ix!();

#[async_trait]
impl<P,H> GenerateReadmeQueries for Workspace<P,H>
where
    // Match the same bounds required by `Workspace` itself and also needed by async context:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + GenerateReadmeQueries<Error = CrateError>
        + ConsolidateCrateInterface
        + Sync
        + Send
        + 'async_trait,
{
    type Error = WorkspaceError;

    #[tracing::instrument(level="trace", skip_all)]
    async fn generate_readme_queries(
        &self,
        _interface: &ConsolidatedCrateInterface
    ) -> Result<Vec<AiReadmeQuery>, Self::Error> {

        trace!("Workspace::generate_readme_queries - building a single workspace-level query");
        // We'll build a short summary of all crates in this workspace, including name + description.

        let mut summary = String::new();
        summary.push_str("This workspace contains the following crates:\n\n");

        // Gather each crate's `name` and `description` from its Cargo.toml
        for (idx, crate_handle) in self.crates().iter().enumerate() {
            // Because `CrateHandleInterface` can give us the `cargo_toml()` method:
            let cargo_toml_arc = crate_handle.cargo_toml();
            // Get the package section; if this fails, we convert to a WorkspaceError
            let package = cargo_toml_arc
                .get_package_section()
                .map_err(|e| WorkspaceError::CrateError(CrateError::CargoTomlError(e)))?;

            // Extract name and description
            let crate_name = package
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown_name>");

            let crate_desc = package
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("<no description provided>");

            // Append to our summary
            summary.push_str(&format!(
                "{}. **Crate:** `{}`\n   **Description:** {}\n\n",
                idx + 1,
                crate_name,
                crate_desc
            ));
        }

        // Now we can build our single AiReadmeQuery using that summary
        let instructions = "We want a top-level README that describes each crate in our workspace.";
        let query_text = format!(
            "We have a workspace with multiple crates. Please generate a README introducing them:\n{}",
            summary
        );

        let query = AiReadmeQueryBuilder::default()
            .query_text(query_text)
            .instructions(instructions)
            .build()
            .unwrap();

        // Return a single query in a Vec
        Ok(vec![query])
    }
}
