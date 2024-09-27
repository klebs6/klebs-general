crate::ix!();

pub struct LintReport {

}

impl Workspace {
    pub fn run_linting(&self) -> Result<LintReport, WorkspaceError> {
        // Run cargo clippy and collect linting results.
        todo!();
    }
}
