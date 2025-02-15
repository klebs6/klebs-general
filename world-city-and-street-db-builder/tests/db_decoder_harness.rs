//-------------------------[tests/db_decoder_harness.rs]
#![allow(unused)]

/////////////////////////////////////////////////////
// Suppose this is tests/db_decoder_harness.rs
// In Cargo.toml:
//    [[test]]
//    name = "db_decoder"
//    harness = false
/////////////////////////////////////////////////////

use world_city_and_street_db_builder::*;
use country::*;
use postal_code::*;
use tempfile::*;
use std::io;
use std::sync::{Mutex,Arc};
use std::{env, process::{Command,ExitStatus}};

/// Our "test" type
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The main entry point. We check if we're in child mode:
///   db_decoder __child_test decode_value_for_key
/// Otherwise, we're the parent harness.
fn main() {
    let mut args = env::args();
    let _bin_name = args.next(); // skip path
    if let Some(subcmd) = args.next() {
        if subcmd == "__child_test" {
            let test_name = args.next().unwrap_or_default();
            let exit_code = child_main_for_test(&test_name);
            std::process::exit(exit_code);
        }
    }

    harness_main();
}

/// The parent harness that spawns children for each test function.
fn harness_main() {
    eprintln!("=== Starting parent harness for db_decoder tests ===");
    let mut failures = 0;

    // We only have one test function in this example, but you can add more:
    if let Err(e) = run_test_in_child("decode_value_for_key") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if failures == 0 {
        eprintln!("All db_decoder tests PASSED");
        std::process::exit(0);
    } else {
        eprintln!("{} test(s) FAILED in db_decoder harness", failures);
        std::process::exit(1);
    }
}

/// Spawns a child process that runs the given test_name.
fn run_test_in_child(test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("--- Spawning child for {} ---", test_name);
    let status: ExitStatus = Command::new(std::env::current_exe()?)
        .arg("__child_test")
        .arg(test_name)
        .status()?;

    if status.success() {
        eprintln!("+++ PASSED (child exited 0): {}\n", test_name);
        Ok(())
    } else {
        Err(format!(
            "Test {} failed in child process (status={:?})",
            test_name, status
        )
        .into())
    }
}

/// Called in the child process to run exactly one "test".
fn child_main_for_test(test_name: &str) -> i32 {
    eprintln!("[child_main_for_test] Running test '{}' in child", test_name);
    match test_name {
        "decode_value_for_key" => {
            let result = database_value_decoder_tests::child_test_decode_value_for_key_all_scenarios();
            if let Err(e) = result {
                eprintln!(
                    "[child_main_for_test] child_test_decode_value_for_key_all_scenarios FAILED: {}",
                    e
                );
                return 1;
            }
            0
        }
        other => {
            eprintln!("[child_main_for_test] ERROR: unknown test name '{}'", other);
            101
        }
    }
}

// --------------------------------------------------------
// Your actual test code is in this module
// --------------------------------------------------------
mod database_value_decoder_tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    /// A small helper to create an in‐memory or temp-dir DB
    fn create_test_db<I: StorageInterface>() -> Arc<Mutex<I>> {
        let tmp = TempDir::new().unwrap();
        I::open(tmp.path()).expect("DB open").clone()
    }

    /// Utility to produce CBOR data for a set of items, stored in a CompressedList.
    fn compress_list_to_cbor<T>(items: Vec<T>) -> Vec<u8>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        let clist = CompressedList::from(items);
        serde_cbor::to_vec(&clist).unwrap()
    }

    // Child test function, returns Result
    pub fn child_test_decode_value_for_key_all_scenarios() -> Result<(), io::Error> {
        eprintln!("[child_test_decode_value_for_key_all_scenarios] STARTING test in child");

        let db_arc = create_test_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let city_cbor = compress_list_to_cbor(vec![
            CityName::new("baltimore").unwrap(),
            CityName::new("frederick").unwrap(),
        ]);
        let street_cbor = compress_list_to_cbor(vec![
            StreetName::new("main st").unwrap(),
            StreetName::new("second ave").unwrap(),
        ]);
        let postal_cbor = compress_list_to_cbor(vec![
            PostalCode::new(Country::USA, "12345").unwrap(),
            PostalCode::new(Country::USA, "99999").unwrap(),
        ]);
        let corrupted_cbor = b"corrupted not valid cbor".to_vec();

        // 1) Z2C => decode as CityName
        let z2c_key = "Z2C:US:12345";
        let out_z2c_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(z2c_key, &city_cbor);
        })?;
        assert!(
            out_z2c_valid.contains(r#"Decoded as Cities: [CityName { name: "baltimore""#),
            "Should decode valid city cbor, but got:\n{}",
            out_z2c_valid
        );

        // Let's see if "corrupted => decode fails => warns" is in stdout
        // We replaced the `warn!` with a plain println below,
        // so it definitely appears in stdout.

        let out_z2c_corrupt = capture_stdout(|| {
            db_guard.decode_value_for_key(z2c_key, &corrupted_cbor);
        })?;
        assert!(
            out_z2c_corrupt.contains("Failed to decode as Cities:"),
            "Corrupted => decode fails => warns,\ngot:\n{}",
            out_z2c_corrupt
        );

        // 2) C2Z => decode as PostalCode
        let c2z_key = "C2Z:US:baltimore";
        let out_c2z_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(c2z_key, &postal_cbor);
        })?;
        assert!(
            out_c2z_valid.contains(r#"Decoded as Postal codes: [PostalCode { country: USA, code: "12345""#),
            "Should decode postal cbor, but got:\n{}",
            out_c2z_valid
        );

        let out_c2z_corrupt = capture_stdout(|| {
            db_guard.decode_value_for_key(c2z_key, &corrupted_cbor);
        })?;
        assert!(
            out_c2z_corrupt.contains("Failed to decode as Postal codes:"),
            "Corrupted => decode fails => warns,\ngot:\n{}",
            out_c2z_corrupt
        );

        // 3) S: => decode as StreetName
        let s_key = "S:US:99999";
        let out_s_valid = capture_stdout(|| {
            db_guard.decode_value_for_key(s_key, &street_cbor);
        })?;
        assert!(
            out_s_valid.contains(r#"Decoded as Streets: [StreetName { name: "main st""#),
            "Decoded as StreetName, but got:\n{}",
            out_s_valid
        );

        // 4) S2C => decode as CityName
        let s2c_key = "S2C:US:main st";
        let out_s2c = capture_stdout(|| {
            db_guard.decode_value_for_key(s2c_key, &city_cbor);
        })?;
        assert!(
            out_s2c.contains(r#"Decoded as Cities: [CityName { name: "baltimore""#),
            "Decode city for S2C, but got:\n{}",
            out_s2c
        );

        // 5) S2Z => decode as PostalCode
        let s2z_key = "S2Z:US:main st";
        let out_s2z = capture_stdout(|| {
            db_guard.decode_value_for_key(s2z_key, &postal_cbor);
        })?;
        assert!(
            out_s2z.contains(r#"Decoded as Postal codes: [PostalCode { country: USA, code: "12345""#),
            "Decode postal for S2Z, but got:\n{}",
            out_s2z
        );

        // 6) C2S => decode as StreetName
        let c2s_key = "C2S:US:baltimore";
        let out_c2s = capture_stdout(|| {
            db_guard.decode_value_for_key(c2s_key, &street_cbor);
        })?;
        assert!(
            out_c2s.contains(r#"Decoded as Streets: [StreetName { name: "main st""#),
            "Decode street for C2S, but got:\n{}",
            out_c2s
        );

        // 7) META:REGION_DONE
        let meta_key = "META:REGION_DONE:US";
        let out_meta = capture_stdout(|| {
            db_guard.decode_value_for_key(meta_key, b"some marker here");
        })?;
        assert!(
            out_meta.contains("Value: REGION DONE MARKER"),
            "Should print region done marker, but got:\n{}",
            out_meta
        );

        // 8) unknown prefix => "[Unknown key pattern]"
        let unknown_key = "XYZ:some unknown prefix";
        let out_unknown = capture_stdout(|| {
            db_guard.decode_value_for_key(unknown_key, b"whatever");
        })?;
        assert!(
            out_unknown.contains("Value: [Unknown key pattern]"),
            "Unknown => fallback message, but got:\n{}",
            out_unknown
        );

        eprintln!("[child_test_decode_value_for_key_all_scenarios] PASS");
        Ok(())
    }
}
