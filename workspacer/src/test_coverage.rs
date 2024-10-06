crate::ix!();

#[async_trait]
impl RunTestsWithCoverage for Workspace {

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
