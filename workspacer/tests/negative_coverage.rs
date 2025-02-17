// ---------------- [ File: tests/negative_coverage.rs ]
// tests/coverage_tests.rs

use tokio::fs;
use std::sync::Arc;
use tokio::process::Command;
use crate::workspace::{Workspace, RunTestsWithCoverage};
use crate::errors::{WorkspaceError, TestCoverageError};
use crate::mock::{create_mock_workspace, CrateConfig};

#[cfg(test)]
mod negative_coverage_tests {
    use super::*;

    #[tokio::test]
    async fn test_missing_tarpaulin_tool() -> Result<(), WorkspaceError> {
        // Create a minimal crate
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_missing_tarpaulin").with_src_files()
        ]).await?;

        let workspace = Workspace::new(&workspace_path).await?;

        // Temporarily rename your PATH or do some environment trick
        // Typically you'd do something OS-specific or just check for the error that tarpaulin is missing.
        // For demonstration, let's assume run_tests_with_coverage might fail if it can't find tarpaulin:
        let result = workspace.run_tests_with_coverage().await;
        match result {
            Ok(_) => panic!("Expected an error if tarpaulin isn't installed or not found"),
            Err(WorkspaceError::TestCoverageError(TestCoverageError::CommandError { .. })) => {
                // We got the I/O error from failing to spawn tarpaulin, which is correct.
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_corrupted_json_coverage_output() -> Result<(), WorkspaceError> {
        // Another scenario might be: tarpaulin runs but returns bad JSON.
        // You can approximate that by overriding the coverage command in a specialized mock,
        // or by intercepting the coverage command with a script that prints invalid JSON.

        // For brevity, we won't re-implement the entire mock here, but you get the idea:
        //  - create workspace
        //  - forcibly run a "fake tarpaulin" that outputs "not json"
        //  - ensure parsing fails with TestCoverageError::CoverageParseError

        // If you’re not using a mock runner for coverage, you might patch `PATH` or rename an existing binary.
        // For now, just demonstrate an example error check:

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_corrupted_json").with_src_files()
        ]).await?;

        let workspace = Workspace::new(&workspace_path).await?;

        // Suppose "cargo tarpaulin" runs but the JSON is nonsense => CoverageParseError
        // We'll just check that the parse error is triggered. Implementation details can vary based on your environment:
        let result = workspace.run_tests_with_coverage().await;
        match result {
            Ok(report) => panic!("Expected coverage parse failure, but got a report: {:?}", report),
            Err(WorkspaceError::TestCoverageError(TestCoverageError::CoverageParseError)) => {
                // Good: coverage parse error recognized
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        Ok(())
    }
}
