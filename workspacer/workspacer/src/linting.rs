// ---------------- [ File: workspacer/src/linting.rs ]
crate::ix!();

#[async_trait]
impl<P,H:CrateHandleInterface<P>> RunLinting for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

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

impl From<std::process::Output> for LintReport {

    fn from(output: std::process::Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }
}

impl MaybeThrow for LintReport {

    type Error = LintingError;

    fn maybe_throw(&self) -> Result<(),Self::Error> {
        if !self.success() {
            return Err(LintingError::UnknownError {
                stderr: Some(self.stderr.clone()),
                stdout: Some(self.stdout.clone()),
            });
        }

        Ok(())
    }
}
