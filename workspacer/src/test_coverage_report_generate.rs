crate::ix!();

impl GenerateReport for TestCoverageCommand {

    type Report = TestCoverageReport;
    type Error  = TestCoverageError;

    fn generate_report(&self) -> Result<Self::Report, Self::Error> {
        // Try to generate the report from text output
        if let Ok(report) = TestCoverageReport::from_maybe_plaintext_coverage_summary(self.stdout()) {
            return Ok(report);
        }

        // If the stdout is empty or contains errors
        if self.is_stdout_empty() || self.has_stderr_errors() {
            return Err(TestCoverageError::CoverageParseError);
        }

        // Parse the JSON output and generate the report
        let coverage_data = self.parse_json_output()?;
        TestCoverageReport::try_from(&coverage_data)
    }
}
