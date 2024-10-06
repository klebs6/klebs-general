crate::ix!();

pub struct TestCoverageReport {
    total_coverage: f32,   // Coverage percentage
    covered_lines:  usize, // Total lines covered
    missed_lines:   usize, // Total lines missed
    total_lines:    usize, // Total lines in the project
}

impl TryFrom<&serde_json::Value> for TestCoverageReport {

    type Error = TestCoverageError;

    /// Constructor that creates a `TestCoverageReport` from JSON data.
    fn try_from(coverage_data: &serde_json::Value) -> Result<Self, Self::Error> {
        let total_lines    = coverage_data["total_lines"].as_u64().unwrap_or(0) as usize;
        let covered_lines  = coverage_data["covered_lines"].as_u64().unwrap_or(0) as usize;
        let total_coverage = coverage_data["total_coverage"].as_f64().unwrap_or(0.0) as f32;
        let missed_lines   = total_lines.saturating_sub(covered_lines);

        // Handle case where no lines were instrumented or coverage is NaN
        if total_lines == 0 || total_coverage.is_nan() {
            error!("No lines were covered or the coverage report was invalid.");
            return Err(TestCoverageError::CoverageParseError);
        }

        Ok(Self::new(total_coverage, covered_lines, missed_lines, total_lines))
    }
}

impl TestCoverageReport {

    /// Creates a new `TestCoverageReport` with given parameters.
    pub fn new(
        total_coverage: f32, 
        covered_lines:  usize, 
        missed_lines:   usize, 
        total_lines:    usize
    ) -> Self {
        Self {
            total_coverage,
            covered_lines,
            missed_lines,
            total_lines,
        }
    }

    /// Constructor that creates a `TestCoverageReport` from a plain text coverage summary.
    pub fn from_maybe_plaintext_coverage_summary(stdout: &str) -> Result<Self, TestCoverageError> {
        let re = Regex::new(r"(\d+\.\d+)% coverage, (\d+)/(\d+) lines covered")
            .map_err(|_| TestCoverageError::CoverageParseError)?;

        if let Some(caps) = re.captures(stdout) {
            let total_coverage = caps[1].parse::<f32>().unwrap_or(0.0);
            let covered_lines  = caps[2].parse::<usize>().unwrap_or(0);
            let total_lines    = caps[3].parse::<usize>().unwrap_or(0);
            let missed_lines   = total_lines.saturating_sub(covered_lines);

            Ok(Self::new(total_coverage, covered_lines, missed_lines, total_lines))
        } else {
            Err(TestCoverageError::CoverageParseError)
        }
    }

    /// Returns the total code coverage as a percentage.
    pub fn total_coverage(&self) -> f32 {
        self.total_coverage
    }

    /// Returns the number of lines covered by tests.
    pub fn covered_lines(&self) -> usize {
        self.covered_lines
    }

    /// Returns the number of lines missed by tests.
    pub fn missed_lines(&self) -> usize {
        self.missed_lines
    }

    /// Returns the total number of lines in the workspace.
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }
}
