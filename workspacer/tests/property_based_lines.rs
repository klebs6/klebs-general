// ---------------- [ File: tests/property_based_lines.rs ]
// tests/property_based_lines.rs
use crate::count_lines_in_file; // Or wherever your function is
use std::path::PathBuf;
use tokio::fs;
use quickcheck::{quickcheck, TestResult};
use crate::errors::WorkspaceError;

fn arb_lines_test() {
    // This function is discovered by quickcheck (not by cargo test directly).
    quickcheck(prop_count_lines as fn(Vec<String>) -> TestResult);
}

/// Property: If we create a temp file with N lines, then `count_lines_in_file` should return N.
fn prop_count_lines(lines: Vec<String>) -> TestResult {
    // If lines is empty, we can choose to skip or we can handle that case
    if lines.is_empty() {
        return TestResult::discard();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        // 1) create temp file
        let temp_dir = std::env::temp_dir().join("quickcheck_prop_test_lines");
        let _ = fs::create_dir_all(&temp_dir).await;
        let file_path = temp_dir.join(format!("lines_{}.txt", uuid::Uuid::new_v4()));

        // 2) write all lines to the file
        let data = lines.join("\n");
        fs::write(&file_path, data).await.map_err(|_| "failed to write file")?;

        // 3) call count_lines_in_file
        let count = count_lines_in_file(&file_path).await.map_err(|_| "failed to count lines")?;

        // 4) check result
        Ok::<usize, &'static str>(count)
    });

    match result {
        Ok(count) => {
            let expected = lines.len();
            TestResult::from_bool(count == expected)
        }
        Err(_) => TestResult::failed(),
    }
}

#[cfg(test)]
mod lines_property_tests {
    use super::*;

    // This is a standard cargo test that triggers the quickcheck property test.
    #[test]
    fn test_count_lines_property() {
        arb_lines_test(); // calls quickcheck
    }
}
