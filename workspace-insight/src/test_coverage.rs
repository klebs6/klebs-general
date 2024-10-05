crate::ix!();

impl Workspace {

    /// Runs tests and gathers code coverage.
    pub async fn run_tests_with_coverage(&self) 
        -> Result<TestCoverageReport, WorkspaceError> 
    {
        let workspace_path = self.path();  // Assuming `self.path()` returns the workspace root path.

        let test_coverage = TestCoverageCommand::run_in(workspace_path).await?;

        let report = test_coverage.generate_report()?;

        Ok(report)
    }
}
