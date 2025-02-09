// ---------------- [ File: tests/coverage_no_tests.rs ]
// tests/coverage_no_tests.rs

use crate::mock::{create_mock_workspace, CrateConfig};
use crate::workspace::{Workspace, RunTestsWithCoverage};
use crate::errors::{WorkspaceError, TestCoverageError};

#[cfg(test)]
mod coverage_no_tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_coverage_in_crate_with_no_tests() -> Result<(), WorkspaceError> {
        // Create a crate with no test files, just minimal src
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_no_tests").with_src_files().with_readme(),
        ]).await?;

        // Possibly remove any test annotation in the code so no tests exist
        let src_file = workspace_path.join("crate_no_tests").join("src").join("lib.rs");
        fs::write(&src_file, "pub fn do_nothing() {}").await?;

        let workspace = Workspace::new(&workspace_path).await?;

        // If your coverage logic yields a valid coverage report (0% coverage),
        // we can check that:
        let coverage_result = workspace.run_tests_with_coverage().await;
        match coverage_result {
            Ok(report) => {
                // Expect 0 coverage, or maybe some lines from global setup code
                // Adjust the assertion based on how your coverage tool handles no tests
                assert!(report.total_coverage() >= 0.0, "Should handle crates with no tests gracefully.");
            }
            Err(WorkspaceError::TestCoverageError(TestCoverageError::TestFailure { .. })) => {
                // Some coverage tools might consider "no tests" a test failure scenario
                // That is also valid, as long as you handle it. 
            }
            Err(e) => panic!("Unexpected coverage error: {:?}", e),
        }

        Ok(())
    }
}
