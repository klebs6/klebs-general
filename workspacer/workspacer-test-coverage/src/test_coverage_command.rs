// ---------------- [ File: workspacer-test-coverage/src/test_coverage_command.rs ]
crate::ix!();

#[derive(Debug)]
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

#[cfg(test)]
mod test_test_coverage_command_real {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use workspacer_3p::tokio::process::Command;
    use workspacer_3p::tokio;

    /// If tests fail, tarpaulin should return a non-zero exit code and mention "Test failed during run".
    #[traced_test]
    async fn test_run_in_test_failure() {
        trace!("Beginning test_run_in_test_failure...");

        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let path = tmp_dir.path();

        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("Failed to spawn `cargo init`");

        if !init_status.status.success() {
            warn!("Skipping test because `cargo init` failed with status: {:?}", init_status.status);
            return;
        }

        // Insert a failing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
        #[test]
        fn test_fail() { assert_eq!(1+1,3); }
        "#;
        tokio::fs::write(&main_rs, code)
            .await
            .expect("Failed to write test code to main.rs");

        // Now run coverage
        let coverage_cmd_result = TestCoverageCommand::run_in(path).await;
        match coverage_cmd_result {
            Err(TestCoverageError::TestFailure { stderr, stdout }) => {
                info!("test_run_in_test_failure stderr: {:?}", stderr);
                info!("test_run_in_test_failure stdout: {:?}", stdout);
            }
            Ok(_) => panic!("Expected coverage to fail on a failing test, but got success"),
            other => panic!("Expected TestFailure, got: {:?}", other),
        }
    }

    #[traced_test]
    async fn test_run_in_succeeds_plaintext_or_json() {
        trace!("Beginning test_run_in_succeeds_plaintext_or_json...");

        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let path = tmp_dir.path();

        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("Failed to spawn `cargo init`");

        if !init_status.status.success() {
            warn!("Skipping test because `cargo init` failed with status: {:?}", init_status.status);
            return;
        }

        // Insert a passing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
        #[test]
        fn test_ok(){ assert_eq!(2+2,4); }
        "#;
        tokio::fs::write(&main_rs, code)
            .await
            .expect("Failed to write test code to main.rs");

        // Run coverage
        let coverage_cmd = match TestCoverageCommand::run_in(path).await {
            Ok(cmd) => cmd,
            Err(e) => panic!("Expected coverage to succeed but got an error: {:?}", e),
        };

        info!("stdout after coverage run:\n{}", coverage_cmd.stdout());
        info!("stderr after coverage run:\n{}", coverage_cmd.stderr());

        // Attempt to parse coverage
        match coverage_cmd.generate_report() {
            Ok(report) => {
                info!("Parsed coverage report: {:?}", report);
                // Additional assertions or checks can be done here
            }
            Err(e) => {
                warn!("Coverage parse failed (could be plain text or JSON not recognized). Error: {:?}", e);
            }
        }
    }
}
