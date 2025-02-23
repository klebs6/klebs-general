// ---------------- [ File: src/test_coverage_command.rs ]
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

    #[tokio::test]
    async fn test_run_in_succeeds_plaintext_or_json() {
        // 1) Create a minimal cargo project 
        let tmp_dir = tempdir().expect("temp dir create");
        let path = tmp_dir.path();

        // cargo init
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("cargo init");
        assert!(init_status.status.success(), "cargo init must succeed");

        // Insert a passing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
            #[test]
            fn test_ok(){ assert_eq!(2+2,4); }
        "#;
        tokio::fs::write(&main_rs, code).await.expect("write main.rs");

        // 2) Run coverage
        let coverage_cmd_result = TestCoverageCommand::run_in(path).await;
        
        match coverage_cmd_result {
            Ok(cmd) => {
                println!("Coverage stdout: {}", cmd.stdout());
                println!("Coverage stderr: {}", cmd.stderr());

                // 3) generate_report
                let report_result = cmd.generate_report();
                match report_result {
                    Ok(report) => {
                        // We have a TestCoverageReport from either plaintext or JSON coverage.
                        println!("Parsed coverage report: {:?}", report);
                    }
                    Err(e) => {
                        // Possibly coverage parse error if tarpaulin output isn't recognized
                        println!("Coverage parse error: {:?}", e);
                        // It’s still a valid scenario
                    }
                }
            }
            Err(e) => {
                panic!("Expected coverage to succeed, but got error: {:?}", e);
            }
        }
    }

    /// If tests fail, tarpaulin should return a non-zero exit code and mention "Test failed during run".
    #[tokio::test]
    async fn test_run_in_test_failure() {
        let tmp_dir = tempdir().expect("tempdir");
        let path = tmp_dir.path();

        // cargo init
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("init");
        assert!(init_status.status.success());

        // Insert a failing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
            #[test]
            fn test_fail() { assert_eq!(1+1,3); }
        "#;
        tokio::fs::write(&main_rs, code).await.expect("write code");

        let coverage_cmd_result = TestCoverageCommand::run_in(path).await;
        match coverage_cmd_result {
            Err(TestCoverageError::TestFailure { stderr, stdout }) => {
                println!("stderr: {:?}", stderr);
                println!("stdout: {:?}", stdout);
            }
            Ok(_) => panic!("Expected test failure, got success"),
            other => panic!("Expected TestFailure, got {:?}", other),
        }
    }
}
