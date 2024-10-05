crate::ix!();

#[derive(Debug)]
pub struct LintReport {
    stdout: String,
    stderr: String,
    success: bool,
}

impl LintReport {

    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    pub fn success(&self) -> bool {
        self.success
    }
}

impl Workspace {

    /// Runs cargo clippy to lint the workspace and collects the linting results.
    pub async fn run_linting(&self) -> Result<LintReport, LintingError> {

        let workspace_path = self.path();  // Assuming `self.path()` returns the workspace root path.

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
            .map_err(|e| LintingError::CommandError { io: e })?;  // Handle any I/O error from the process execution.

        // Capture the linting results in LintReport.
        let report = LintReport {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        };

        // If clippy failed, return an error.
        if !report.success {
            return Err(LintingError::UnknownError {
                stderr: Some(report.stderr.clone()),
                stdout: Some(report.stdout.clone()),
            });
        }

        Ok(report)  // Return the linting report if successful.
    }
}
