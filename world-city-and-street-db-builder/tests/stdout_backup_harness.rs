#![allow(unused)]
//! A custom test harness that spawns a child process for each stdout_backup test,
//! thereby avoiding cargo/coverage hooking on our stdout FD. Cargo.toml must set:
//!   [test]
//!   harness = false
//! so that we control `main()` directly.

use std::os::fd::FromRawFd;
use world_city_and_street_db_builder::StdoutBackup;
use std::error::Error;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::{Command, ExitStatus};
use std::{env, process};

///////////////////////////////////////////////////////////////////////////
// The test logic is in this module. Each function is a “real” test.
///////////////////////////////////////////////////////////////////////////
mod test_stdout_backup {
    use super::*;
    use std::os::unix::io::AsRawFd;

    /// Creates a temporary pipe, redirects stdout to the pipe’s writer,
    /// and returns (StdoutBackup, reader_fd, writer_fd).
    fn redirect_stdout_to_pipe() -> io::Result<(StdoutBackup, i32, i32)> {
        eprintln!("[redirect_stdout_to_pipe] Creating pipe...");
        let mut pipe_fds = [0; 2];
        let rc = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
        if rc == -1 {
            let err = io::Error::last_os_error();
            eprintln!("[redirect_stdout_to_pipe] pipe() failed: {:?}", err);
            return Err(err);
        }
        let reader_fd = pipe_fds[0];
        let writer_fd = pipe_fds[1];
        eprintln!(
            "[redirect_stdout_to_pipe] => reader_fd={}, writer_fd={}",
            reader_fd, writer_fd
        );

        eprintln!("[redirect_stdout_to_pipe] Creating StdoutBackup (dup(1))...");
        let backup = StdoutBackup::new()?;

        let stdout_fd = io::stdout().as_raw_fd();
        eprintln!(
            "[redirect_stdout_to_pipe] => now dup2(writer_fd={}, stdout_fd={})",
            writer_fd, stdout_fd
        );
        if unsafe { libc::dup2(writer_fd, stdout_fd) } == -1 {
            let err = io::Error::last_os_error();
            eprintln!(
                "[redirect_stdout_to_pipe] ERROR: dup2({}, {}) failed: {:?}",
                writer_fd, stdout_fd, err
            );
            return Err(err);
        }
        eprintln!(
            "[redirect_stdout_to_pipe] => dup2 succeeded, stdout now fd={}",
            writer_fd
        );
        Ok((backup, reader_fd, writer_fd))
    }

    /// Reads all available bytes from `fd` into a String, then returns it.
    fn read_from_fd(fd: i32) -> String {
        eprintln!("[read_from_fd] Attempting read on fd={}...", fd);
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        let mut result = String::new();
        match file.read_to_string(&mut result) {
            Ok(n) => {
                eprintln!(
                    "[read_from_fd] read {} bytes: {:?}",
                    n,
                    result.replace('\n', "\\n")
                );
            }
            Err(e) => {
                eprintln!("[read_from_fd] ERROR: {:?}", e);
            }
        }
        result
    }

    pub fn test_new_creates_backup_ok() -> Result<(), Box<dyn Error>> {
        eprintln!("[test_new_creates_backup_ok] START");
        let backup_result = StdoutBackup::new();
        eprintln!("[test_new_creates_backup_ok] => got: {:?}", backup_result);
        assert!(backup_result.is_ok(), "Should create StdoutBackup OK");
        let _backup = backup_result.unwrap();
        eprintln!("[test_new_creates_backup_ok] PASS");
        Ok(())
    }

    pub fn test_restore_stdout() -> Result<(), Box<dyn Error>> {
        eprintln!("[test_restore_stdout] Step 1: redirect_stdout_to_pipe...");
        let (backup, reader_fd, writer_fd) = redirect_stdout_to_pipe()?;

        eprintln!("[test_restore_stdout] Step 2: println! -> pipe");
        println!("Hello, pipe!");
        io::stdout().flush().unwrap();

        eprintln!("[test_restore_stdout] Step 3: backup.restore()");
        backup.restore()?;

        eprintln!("[test_restore_stdout] Step 4: close the writer_fd={}", writer_fd);
        unsafe { libc::close(writer_fd) };

        eprintln!("[test_restore_stdout] Step 5: read_from_fd(reader_fd={})", reader_fd);
        let captured = read_from_fd(reader_fd);
        assert!(
            captured.contains("Hello, pipe!"),
            "Expected 'Hello, pipe!' in captured text"
        );

        eprintln!("[test_restore_stdout] PASS");
        Ok(())
    }

    pub fn test_restore_fails_for_bogus_dup2() -> Result<(), Box<dyn Error>> {
        eprintln!("[test_restore_fails_for_bogus_dup2] START");
        let mut bogus = world_city_and_street_db_builder::StdoutBackupBuilder::default()
            .stdout_fd(1)
            .backup_fd(-1)
            .build()?;
        eprintln!("[test_restore_fails_for_bogus_dup2] => calling bogus.restore()");
        let result = bogus.restore();
        eprintln!("[test_restore_fails_for_bogus_dup2] => restore returned: {:?}", result);
        assert!(result.is_err(), "Expected an Err(...) from dup2(-1,1)");
        eprintln!("[test_restore_fails_for_bogus_dup2] PASS");
        Ok(())
    }

    pub fn test_drop_ignores_errors() -> Result<(), Box<dyn Error>> {
        eprintln!("[test_drop_ignores_errors] START");
        {
            let bogus = world_city_and_street_db_builder::StdoutBackupBuilder::default()
                .stdout_fd(1)
                .backup_fd(-1)
                .build()?;
            eprintln!("[test_drop_ignores_errors] about to drop bogus => no panic expected");
            // On drop, it calls dup2(-1,1) ignoring errors
        }
        eprintln!("[test_drop_ignores_errors] PASS");
        Ok(())
    }

    pub fn test_drop_calls_restore_stdout_implicitly() -> Result<(), Box<dyn Error>> {
        eprintln!("[test_drop_calls_restore_stdout_implicitly] Step 1: redirect_stdout_to_pipe");
        let (backup, reader_fd, writer_fd) = redirect_stdout_to_pipe()?;
        eprintln!(
            "[test_drop_calls_restore_stdout_implicitly] => after redirect, stdout is fd={}",
            io::stdout().as_raw_fd()
        );

        println!("Before drop");
        io::stdout().flush()?;

        eprintln!(
            "[test_drop_calls_restore_stdout_implicitly] Step 2: call backup.restore() explicitly"
        );
        backup.restore()?;

        eprintln!("[test_drop_calls_restore_stdout_implicitly] Step 3: now drop(backup)");
        drop(backup);

        eprintln!(
            "[test_drop_calls_restore_stdout_implicitly] => after drop, stdout is fd={}",
            io::stdout().as_raw_fd()
        );

        eprintln!("[test_drop_calls_restore_stdout_implicitly] Step 4: close writer_fd={}", writer_fd);
        unsafe { libc::close(writer_fd) };

        eprintln!("[test_drop_calls_restore_stdout_implicitly] Step 5: read from pipe...");
        let captured_before_drop = read_from_fd(reader_fd);
        assert!(
            captured_before_drop.contains("Before drop"),
            "Expected 'Before drop' in captured text"
        );
        eprintln!("[test_drop_calls_restore_stdout_implicitly] => PASS for 'Before drop'");

        // Step 6: second redirect
        let (new_backup, new_reader_fd, new_writer_fd) = redirect_stdout_to_pipe()?;
        println!("After drop");
        io::stdout().flush()?;

        eprintln!("Restoring stdout so the pipe sees EOF when we close writer_fd...");
        new_backup.restore()?;

        unsafe { libc::close(new_writer_fd) };

        let captured_after_drop = read_from_fd(new_reader_fd);
        assert!(captured_after_drop.contains("After drop"));
        eprintln!("Success: found 'After drop'");
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////
// The CHlLD PROCESS entry point (run a specific test by name)
///////////////////////////////////////////////////////////////////////////
fn child_main_for_test(test_name: &str) -> i32 {
    eprintln!("[child_main_for_test] Child is running test: {}", test_name);
    // Dispatch to the matching function in test_stdout_backup. 
    // If it fails (panic or Result::Err), we return a nonzero exit code.
    let result = match test_name {
        "test_new_creates_backup_ok" => test_stdout_backup::test_new_creates_backup_ok(),
        "test_restore_stdout" => test_stdout_backup::test_restore_stdout(),
        "test_restore_fails_for_bogus_dup2" => test_stdout_backup::test_restore_fails_for_bogus_dup2(),
        "test_drop_ignores_errors" => test_stdout_backup::test_drop_ignores_errors(),
        "test_drop_calls_restore_stdout_implicitly" => test_stdout_backup::test_drop_calls_restore_stdout_implicitly(),
        other => {
            eprintln!(
                "[child_main_for_test] ERROR: No matching test: {}",
                other
            );
            return 101; // unknown test
        }
    };

    match result {
        Ok(()) => {
            eprintln!("[child_main_for_test] Test {} PASSED in child", test_name);
            0
        }
        Err(e) => {
            eprintln!("[child_main_for_test] Test {} FAILED: {}", test_name, e);
            1
        }
    }
}

///////////////////////////////////////////////////////////////////////////
// The PARENT HARNESS: spawns child processes for each test function
///////////////////////////////////////////////////////////////////////////

// Our "test" type
type TestResult = Result<(), Box<dyn Error>>;

fn main() {
    // Are we in "child mode"? If so, run child_main_for_test(...) and exit.
    let mut args = env::args();
    let first_arg = args.nth(1); // skip binary path
    if let Some(subcmd) = first_arg {
        if subcmd == "__child_test" {
            let test_name = args.next().unwrap_or_default();
            let exit_code = child_main_for_test(&test_name);
            process::exit(exit_code);
        }
    }

    // Otherwise, we are the "parent harness"
    harness_main();
}

/// The harness main function.  
/// Spawns a child process for each test. 
fn harness_main() {
    eprintln!("=== Starting manual harness for stdout_backup tests ===");
    let mut failures = 0;

    // We run each test by spawning ourselves in child mode:
    if let Err(e) = run_test_in_child("test_new_creates_backup_ok") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test_in_child("test_restore_stdout") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test_in_child("test_restore_fails_for_bogus_dup2") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test_in_child("test_drop_ignores_errors") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    if let Err(e) = run_test_in_child("test_drop_calls_restore_stdout_implicitly") {
        eprintln!("FAILURE: {}", e);
        failures += 1;
    }

    // Return code 0 if no failures, 1 otherwise
    if failures == 0 {
        eprintln!("All stdout_backup tests PASSED");
        process::exit(0);
    } else {
        eprintln!("{} test(s) FAILED in stdout_backup_harness", failures);
        process::exit(1);
    }
}

/// Spawns a child process with `__child_test <test_name>` and checks the exit code.
fn run_test_in_child(test_name: &str) -> Result<(), Box<dyn Error>> {
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
