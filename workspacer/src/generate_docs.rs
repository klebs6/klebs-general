crate::ix!();

#[async_trait]
impl GenerateDocs for Workspace {

    type Error = WorkspaceError;

    /// Generates the documentation for the entire workspace by running `cargo doc`.
    async fn generate_docs(&self) -> Result<(), WorkspaceError> {
        let workspace_path = self.as_ref();  // Assuming `self.path()` returns the workspace root path.
        
        // Execute `cargo doc` in the workspace directory.
        let output = Command::new("cargo")
            .arg("doc")
            .current_dir(workspace_path)
            .output()
            .await
            .map_err(|e| CargoDocError::CommandError { io: e.into() })?;  // Handle any I/O error from the process execution.

        if !output.status.success() {
            // If the command failed, return an error with the captured output.
            return Err(WorkspaceError::from(CargoDocError::UnknownError {
                stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            }));
        }

        Ok(())  // If the command was successful, return Ok.
    }
}
