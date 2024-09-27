crate::ix!();

pub struct TestCoverageReport {

}

impl Workspace {
    pub fn run_tests_with_coverage(&self) -> Result<TestCoverageReport, WorkspaceError> {
        // Run tests and gather code coverage.
        todo!();
    }
}
