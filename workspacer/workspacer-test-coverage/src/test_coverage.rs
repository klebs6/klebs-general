// ---------------- [ File: src/test_coverage.rs ]
crate::ix!();

#[async_trait]
pub trait RunTestsWithCoverage {

    type Report;
    type Error;

    async fn run_tests_with_coverage(&self) 
        -> Result<Self::Report, Self::Error>;
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

    // We'll define a trivial coverage command or rely on your real TestCoverageCommand 
    // that spawns coverage tool.

    #[tokio::test]
    async fn test_run_tests_with_coverage_succeeds() {
        // 1) Create a minimal cargo project with a test
        let tmp_dir = tempdir().expect("failed to create temp dir");
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

        // Insert a test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main() {}
            #[test]
            fn test_ok(){ assert_eq!(2+2,4); }
        "#;
        tokio::fs::write(&main_rs, code)
            .await
            .expect("write main.rs");

        // 2) Construct your workspace
        let ws = MockWorkspace { path: path.to_path_buf() };

        // 3) run coverage
        let result = ws.run_tests_with_coverage().await;

        // 4) If the coverage tool is installed and everything passes, we get Ok(report).
        // We can do partial checks on coverage % or lines if your `TestCoverageReport` has them.
        match result {
            Ok(report) => {
                // e.g., check if report.total_coverage() is >= some threshold
                println!("Coverage report: {:?}", report);
            }
            Err(e) => panic!("Coverage run failed: {:?}", e),
        }
    }

    /// If the coverage tool or tests fail, we expect an error.
    #[tokio::test]
    async fn test_run_tests_with_coverage_fails() {
        // Possibly break the code so tests fail or coverage tool fails
        let tmp_dir = tempdir().expect("tempdir");
        let path = tmp_dir.path();

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
            fn main(){}
            #[test]
            fn test_fail(){ assert_eq!(1+1,3); }
        "#;
        tokio::fs::write(&main_rs, code).await.expect("write code");

        let ws = MockWorkspace { path: path.to_path_buf() };

        let result = ws.run_tests_with_coverage().await;
        // Expect some coverage or test error
        match result {
            Err(e) => {
                println!("Coverage or test fail as expected: {:?}", e);
            }
            Ok(r) => panic!("Expected coverage to fail, got success: {:?}", r),
        }
    }
}
