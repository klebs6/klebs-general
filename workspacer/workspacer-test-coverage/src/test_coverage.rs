// ---------------- [ File: workspacer-test-coverage/src/test_coverage.rs ]
crate::ix!();

#[async_trait]
pub trait RunTestsWithCoverage {

    type Report;
    type Error;

    async fn run_tests_with_coverage(&self) 
        -> Result<Self::Report, Self::Error>;
}

// We'll show a naive approach: run "cargo tarpaulin --package crate_name" from the workspace root.
// Then parse stdout => produce a TestCoverageReport
//
#[async_trait]
impl RunTestsWithCoverage for CrateHandle {
    type Report = TestCoverageReport;
    type Error  = WorkspaceError;

    async fn run_tests_with_coverage(&self) -> Result<Self::Report, Self::Error> {
        let workspace_root = self
            .root_dir_path_buf()
            .parent()
            .ok_or_else(|| {
                // If the crate root is the workspace, you might do something else. 
                // We'll do a naive approach for demonstration.
                error!("Cannot get parent directory for crate_path={:?}", self.root_dir_path_buf());
                WorkspaceError::IoError {
                    io_error: std::sync::Arc::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound, 
                        "No parent directory"
                    )),
                    context: "finding workspace root from crate path".to_string(),
                }
            })?
            .to_path_buf();

        // We'll run cargo tarpaulin with `--package crate_name`
        let crate_name = self.name(); 
        let coverage_cmd = TestCoverageCommand::run_with_package(&workspace_root, &crate_name).await?;
        let report = coverage_cmd.generate_report()?;
        Ok(report)
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> RunTestsWithCoverage for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Report = TestCoverageReport;
    type Error  = WorkspaceError;

    /// Runs tests and gathers code coverage.
    async fn run_tests_with_coverage(&self) 
        -> Result<Self::Report, Self::Error> 
    {
        let workspace_path = self.as_ref();  // Assuming `self.path()` returns the workspace root path.

        let test_coverage = TestCoverageCommand::run_in(workspace_path).await?;

        let report = test_coverage.generate_report()?;

        Ok(report)
    }
}

#[cfg(test)]
mod test_run_tests_with_coverage_real {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use workspacer_3p::tokio::process::Command;
    use workspacer_3p::tokio;

    // We'll define or import your real `Workspace<P,H>` and `RunTestsWithCoverage` trait:
    // use crate::{Workspace, RunTestsWithCoverage, TestCoverageReport, TestCoverageCommand, ...};

    /// A minimal mock or real workspace object that implements your coverage method
    #[derive(Debug)]
    struct MockWorkspace {
        path: PathBuf,
    }

    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.path
        }
    }

    // If you have a real `Workspace<P,H>::new(...)`, you can skip the mock struct.

    // Suppose we replicate or rely on your posted code:
    #[async_trait]
    impl RunTestsWithCoverage for MockWorkspace {
        type Report = TestCoverageReport;
        type Error = WorkspaceError;

        async fn run_tests_with_coverage(&self) -> Result<Self::Report, Self::Error> {
            let workspace_path = self.as_ref();
            let test_coverage = TestCoverageCommand::run_in(workspace_path).await?;
            let report = test_coverage.generate_report()?;
            Ok(report)
        }
    }

    #[traced_test]
    async fn test_run_tests_with_coverage_succeeds() {
        trace!("Beginning test_run_tests_with_coverage_succeeds...");

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

        // Insert a small passing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
        #[test]
        fn test_ok() { assert_eq!(2+2,4); }
        "#;
        tokio::fs::write(&main_rs, code)
            .await
            .expect("Failed to write test code to main.rs");

        let ws = MockWorkspace { path: path.to_path_buf() };

        // Running tests with coverage should succeed
        match ws.run_tests_with_coverage().await {
            Ok(report) => {
                info!("Coverage succeeded: {:?}", report);
                assert!(report.covered_lines() > 0, "Expected at least one covered line");
                assert!(report.total_coverage() > 0.0, "Expected non-zero coverage");
            }
            Err(e) => panic!("Expected coverage to succeed, but got error: {:?}", e),
        }
    }

    #[traced_test]
    async fn test_run_tests_with_coverage_fails() {
        trace!("Beginning test_run_tests_with_coverage_fails...");

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
            fn main(){}
        #[test]
        fn test_fail(){ assert_eq!(1+1,3); }
        "#;
        tokio::fs::write(&main_rs, code)
            .await
            .expect("Failed to write test code to main.rs");

        let ws = MockWorkspace { path: path.to_path_buf() };

        // We expect coverage or test to fail
        match ws.run_tests_with_coverage().await {
            Ok(report) => panic!("Expected coverage to fail, got success: {:?}", report),
            Err(e) => {
                info!("Coverage or test failure as expected: {:?}", e);
            }
        }
    }
}
