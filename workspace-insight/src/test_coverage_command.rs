crate::ix!();

pub struct TestCoverageCommand {
    stdout: String,
    stderr: String,
}

impl TestCoverageCommand {

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

impl GenerateReport for TestCoverageCommand {

    type Report = TestCoverageReport;
    type Error  = TestCoverageError;

    fn generate_report(&self) -> Result<Self::Report,Self::Error> {

        // Check if the output contains a plain text coverage summary
        let re = Regex::new(r"(\d+\.\d+)% coverage, (\d+)/(\d+) lines covered").unwrap();

        if let Some(caps) = re.captures(&self.stdout) {
            let total_coverage = caps[1].parse::<f32>().unwrap_or(0.0);
            let covered_lines  = caps[2].parse::<usize>().unwrap_or(0);
            let total_lines    = caps[3].parse::<usize>().unwrap_or(0);
            let missed_lines   = total_lines.saturating_sub(covered_lines);

            return Ok(TestCoverageReport::new(total_coverage, covered_lines, missed_lines, total_lines));
        }

        // If stdout is empty or contains invalid coverage data
        if self.stdout.trim().is_empty() || self.stderr.contains("error") {
            return Err(TestCoverageError::CoverageParseError);
        }

        // Parse the JSON output from tarpaulin to extract coverage data (if present)
        let coverage_data: serde_json::Value = serde_json::from_str(&self.stdout).map_err(|e| {
            error!("Failed to parse coverage report: {}", e);
            TestCoverageError::CoverageParseError
        })?;

        // Extract relevant data from the JSON response
        let total_lines    = coverage_data["total_lines"].as_u64().unwrap_or(0) as usize;
        let covered_lines  = coverage_data["covered_lines"].as_u64().unwrap_or(0) as usize;
        let total_coverage = coverage_data["total_coverage"].as_f64().unwrap_or(0.0) as f32;
        let missed_lines   = total_lines.saturating_sub(covered_lines);

        // Handle case where no lines were instrumented or coverage is NaN
        if total_lines == 0 || total_coverage.is_nan() {
            error!("No lines were covered or the coverage report was invalid.");
            return Err(TestCoverageError::CoverageParseError);
        }

        // Return the TestCoverageReport
        Ok(TestCoverageReport::new(total_coverage, covered_lines, missed_lines, total_lines))
    }
}
