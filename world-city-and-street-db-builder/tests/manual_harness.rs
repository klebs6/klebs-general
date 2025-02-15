//! tests/manual_harness.rs
//!
//! A custom test harness that runs the “capture_stdout” tests manually,
//! printing results directly to stdout. 
//! Because we set `harness = false` in Cargo.toml, Cargo will just run `main()`
//! without capturing anything.

use std::process;

/// If your capture_stdout tests are in `crate::capture_stdout_tests`, import them here:
/// (You might need `pub mod capture_stdout_tests;` or so, or just define
/// test-like functions that we call manually.)
///
/// We'll assume you've moved or re-exported the functions we want to test.

use world_city_and_street_db_builder::*;
use capture_stdout_tests::*; 

mod capture_stdout_tests {
    use super::*;
    use std::error::Error;
    use std::io::Write; // for handle.write_all

    // If you want to avoid the global `--test-threads=1` approach,
    // apply the `#[serial_test::serial]` attribute to each test like so:
    // use serial_test::serial;
    pub fn test_capture_stdout_empty_closure() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {})?;
        assert_eq!(captured, "");
        Ok(())
    }

    pub fn test_capture_stdout_single_line() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {
            println!("Hello, test!");
        })?;
        assert_eq!(captured.trim(), "Hello, test!");
        Ok(())
    }

    pub fn test_capture_stdout_multiple_lines() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {
            println!("Line one");
            println!("Line two");
            println!("Line three");
        })?;
        // `println!` adds a newline after each line => 
        // "Line one\nLine two\nLine three\n"
        assert_eq!(captured, "Line one\nLine two\nLine three\n");
        Ok(())
    }

    pub fn test_capture_stdout_large_output() -> Result<(), Box<dyn Error>> {
        let large_data = "a".repeat(8192);
        let captured = capture_stdout(|| {
            print!("{}", large_data);
        })?;
        assert_eq!(captured.len(), 8192);
        assert!(captured.chars().all(|c| c == 'a'));
        Ok(())
    }

    pub fn test_capture_stdout_non_utf8() -> Result<(), Box<dyn Error>> {
        let bad_bytes = [0x66, 0x6F, 0x80, 0xFE, 0xFF]; // "fo" + some invalid UTF-8
        let captured = capture_stdout(|| {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&bad_bytes).unwrap();
        })?;
        // "fo" remains, invalid bytes become "\u{FFFD}" in .from_utf8_lossy()
        // e.g. "fo���"
        assert!(captured.starts_with("fo"));
        assert!(captured.contains('\u{FFFD}'));
        Ok(())
    }

    pub fn test_capture_stdout_closure_panics() -> Result<(),Box<dyn Error>> {
        use std::panic;
        let result = panic::catch_unwind(|| {
            let _ = capture_stdout(|| {
                println!("This prints before panicking");
                panic!("Simulated panic");
            });
        });
        assert!(result.is_err(), "The closure intentionally panics");
        // If the global lock and backup logic is correct, stdout is restored
        println!("Should still see real stdout here after the panic");
        Ok(())
    }
}

/// The return type from a "test" function
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Our custom test harness
fn main() {
    println!("=== Starting manual_harness main ===");

    let mut failures = 0;

    // Example calls. For each “test” function, run it and handle errors:
    if let Err(e) = run_test("test_capture_stdout_empty_closure", test_capture_stdout_empty_closure) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test("test_capture_stdout_single_line", test_capture_stdout_single_line) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test("test_capture_stdout_multiple_lines", test_capture_stdout_multiple_lines) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test("test_capture_stdout_large_output", test_capture_stdout_large_output) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test("test_capture_stdout_non_utf8", test_capture_stdout_non_utf8) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test("test_capture_stdout_closure_panics", test_capture_stdout_closure_panics) {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    // etc. for all your test_capture_stdout_* functions

    if failures == 0 {
        println!("All manual_harness tests PASSED");
        process::exit(0);
    } else {
        println!("{} tests FAILED in manual_harness", failures);
        // Nonzero exit code => Cargo sees a test failure
        process::exit(1);
    }
}

/// Helper that calls an individual test function, printing its name and capturing errors.
fn run_test(test_name: &str, f: fn() -> TestResult) -> TestResult {
    println!("--- Running {} ---", test_name);
    match f() {
        Ok(()) => {
            println!("+++ PASSED: {}\n", test_name);
            Ok(())
        }
        Err(e) => {
            Err(format!("Test {} failed with error: {}", test_name, e).into())
        }
    }
}
