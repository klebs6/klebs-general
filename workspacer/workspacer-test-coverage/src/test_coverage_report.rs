// ---------------- [ File: src/test_coverage_report.rs ]
crate::ix!();

#[derive(Debug)]
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

#[cfg(test)]
#[disable]
mod test_coverage_integration {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use workspacer_3p::tokio::process::Command;
    use workspacer_3p::tokio;

    #[tokio::test]
    async fn test_parse_coverage_report_from_real_tarpaulin_run() {
        // ---------------------------------------------------------------------
        // 1) Prepare a minimal Cargo project in a temp directory
        // ---------------------------------------------------------------------
        let tmp_dir = tempdir().expect("Failed to create temp directory for test");
        let project_path = tmp_dir.path();

        // Run `cargo init --bin --vcs none` to create a basic binary crate
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(project_path)
            .output()
            .await
            .expect("Failed to run `cargo init`");
        assert!(
            init_status.status.success(),
            "cargo init must succeed in order to proceed"
        );

        // Write a small main.rs with at least one test
        let src_main = project_path.join("src").join("main.rs");
        let code = r#"
            fn main() {
                println!("Hello, coverage!");
            }

            #[test]
            fn test_ok() {
                assert_eq!(2 + 2, 4);
            }
        "#;
        tokio::fs::write(&src_main, code)
            .await
            .expect("Failed to write main.rs test code");

        // ---------------------------------------------------------------------
        // 2) Actually run `cargo tarpaulin --out Json --quiet` in that directory
        // ---------------------------------------------------------------------
        // Make sure tarpaulin is installed or the test will fail.
        let output = Command::new("cargo")
            .arg("tarpaulin")
            .arg("--out")
            .arg("Json")
            .arg("--")
            .arg("--quiet")
            .current_dir(&project_path)
            .output()
            .await;

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                // If tarpaulin isn't installed or the spawn fails, handle it:
                panic!("Failed to run cargo tarpaulin: {}", e);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        eprintln!("tarpaulin stdout:\n{}", stdout);
        eprintln!("tarpaulin stderr:\n{}", stderr);

        // If tarpaulin returned non-zero, check if tests failed or coverage had issues.
        if !output.status.success() {
            // Possibly the test or coverage failed. For a healthy scenario, we expect success:
            panic!("Coverage tool failed. stderr:\n{}", stderr);
        }

        // ---------------------------------------------------------------------
        // 3) Construct our TestCoverageCommand object and parse
        // ---------------------------------------------------------------------
        let coverage_cmd = TestCoverageCommand {
            stdout,
            stderr,
        };

        // If your coverage tool sometimes prints a plaintext summary (like "80.0% coverage, 8/10 lines covered")
        // you can rely on `from_maybe_plaintext_coverage_summary`. 
        // If it prints JSON, your code tries `parse_json_output` => `try_from(&json)`.
        // The `generate_report()` internally tries plaintext first, then JSON. 
        // So we just do:
        let coverage_report = match coverage_cmd.generate_report() {
            Ok(report) => report,
            Err(e) => panic!("Failed to parse coverage report: {:?}", e),
        };

        // ---------------------------------------------------------------------
        // 4) Inspect the final TestCoverageReport
        // ---------------------------------------------------------------------
        eprintln!("Parsed coverage report:\n  total_coverage:   {}%\n  covered_lines:    {}\n  missed_lines:     {}\n  total_lines:      {}",
            coverage_report.total_coverage(),
            coverage_report.covered_lines(),
            coverage_report.missed_lines(),
            coverage_report.total_lines(),
        );

        // Some minimal assertion: we expect at least 1 covered line
        assert!(
            coverage_report.covered_lines() > 0,
            "We expected at least one covered line in this project"
        );
        assert!(
            coverage_report.total_coverage() > 0.0,
            "We expected some non-zero coverage"
        );
    }
}
