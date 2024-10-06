crate::ix!();

#[async_trait]
impl RunLinting for Workspace {

    type Report = LintReport;
    type Error  = LintingError;

    /// Runs cargo clippy to lint the workspace and collects the linting results.
    async fn run_linting(&self) -> Result<Self::Report, Self::Error> {

        let workspace_path = self.as_ref();  // Assuming `self.path()` returns the workspace root path.

        // Run `cargo clippy` in the workspace directory, treating warnings as errors.
        let output = tokio::process::Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--message-format=short")
            .arg("--quiet")
            .arg("--")
            .arg("-D")
            .arg("warnings")  // Deny warnings to force failure on lint issues
            .current_dir(workspace_path)
            .output()
            .await
            .map_err(|e| LintingError::CommandError { io: e.into() })?;  // Handle any I/O error from the process execution.

        // Capture the linting results in LintReport.
        let report = LintReport::from(output);

        // If clippy failed, return an error.
        report.maybe_throw()?;

        Ok(report)  // Return the linting report if successful.
    }
}
