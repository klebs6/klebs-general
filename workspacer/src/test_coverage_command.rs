crate::ix!();

pub struct TestCoverageCommand {
    stdout: String,
    stderr: String,
}

impl TestCoverageCommand {

    // Add a method to access stdout
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    // Add a method to access stderr
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    // Add a method to check if stdout is empty
    pub fn is_stdout_empty(&self) -> bool {
        self.stdout.trim().is_empty()
    }

    // Add a method to check if stderr contains errors
    pub fn has_stderr_errors(&self) -> bool {
        self.stderr.contains("error")
    }

    // Add a method to parse JSON output
    pub fn parse_json_output(&self) -> Result<serde_json::Value, TestCoverageError> {
        serde_json::from_str(&self.stdout).map_err(|e| {
            error!("Failed to parse coverage report: {}", e);
            TestCoverageError::CoverageParseError
        })
    }

    pub async fn run_in(workspace_path: impl AsRef<Path>) 
        -> Result<Self,TestCoverageError> 
    {
        // Run `cargo tarpaulin` in the workspace directory to collect test coverage
        let output = tokio::process::Command::new("cargo")
            .arg("tarpaulin")
            .arg("--out")
            .arg("Json")
            .arg("--")
            .arg("--quiet")
            .current_dir(workspace_path)
            .output()
            .await
            .map_err(|e| TestCoverageError::CommandError { io: e.into() })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Log the stdout and stderr for debugging purposes
        info!("stdout: {}", stdout);
        info!("stderr: {}", stderr);

        match output.status.success() {

            true => Ok(Self { stdout, stderr }),

            // If the tarpaulin command failed, check if the failure was due to tests failing
            false => {
                if stderr.contains("Test failed during run") {
                    error!("Test coverage run failed due to test failure.");
                    Err(TestCoverageError::TestFailure {
                        stderr: Some(stderr),
                        stdout: Some(stdout),
                    })
                } else {
                    error!("Test coverage run failed for unknown reasons.");
                    Err(TestCoverageError::UnknownError { 
                        stdout: Some(stdout),
                        stderr: Some(stderr), 
                    })
                }
            }
        }
    }
}
