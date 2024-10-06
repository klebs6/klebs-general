crate::ix!();

pub struct TestCoverageReport {
    total_coverage: f32,   // Coverage percentage
    covered_lines:  usize, // Total lines covered
    missed_lines:   usize, // Total lines missed
    total_lines:    usize, // Total lines in the project
}

impl TestCoverageReport {

    /// Creates a new `TestCoverageReport`
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
