// ---------------- [ File: src/capture_stdout.rs ]
// ---------------- [ File: src/capture_stdout.rs ]
crate::ix!();

use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

/// Captures everything printed to stdout during the execution of the given closure.
///
/// This function creates an OS pipe and redirects stdout to the write end of that pipe.
/// After the closure is executed, stdout is restored and the contents of the pipe are
/// read and returned as a `String`.
///
/// # Arguments
///
/// * `f` - A closure that performs printing to stdout.
///
/// # Returns
///
/// * `Ok(String)` containing the captured output on success.
/// * An `io::Error` if any system call (dup, dup2, pipe, or I/O operation) fails.
///
/// # Safety
///
/// This function uses unsafe code to manipulate file descriptors via libc functions.
///
pub fn capture_stdout<F: FnOnce()>(f: F) -> io::Result<String> {
    // Save the original stdout.
    let backup = StdoutBackup::new()?;

    // Create a pipe. The write end (writer) will be used to capture stdout,
    // while the read end (reader) will be used later to collect the output.
    let (mut reader, writer) = os_pipe::pipe()?;

    let stdout_fd = io::stdout().as_raw_fd();

    // Redirect stdout to the pipe's writer.
    if unsafe { libc::dup2(writer.as_raw_fd(), stdout_fd) } == -1 {
        return Err(io::Error::last_os_error());
    }

    // Execute the closure that writes to stdout.
    f();

    // Flush stdout to ensure all buffered output is written to the pipe.
    io::stdout().flush()?;

    // Restore the original stdout using the backup.
    backup.restore()?;

    // Drop the writer to signal EOF on the reading side of the pipe.
    drop(writer);

    // Read everything from the pipe.
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[cfg(test)]
pub mod capture_stdout_tests {
    use super::*;

    /// Ensures that capturing stdout from an empty closure yields an empty string.
    #[traced_test]
    fn test_capture_stdout_empty_closure() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {})?;
        assert_eq!(captured, "");
        Ok(())
    }

    /// Verifies that a single-line print statement is captured correctly.
    #[traced_test]
    fn test_capture_stdout_single_line() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {
            println!("Hello, test!");
        })?;
        // Use `trim()` to remove the trailing newline introduced by `println!`.
        assert_eq!(captured.trim(), "Hello, test!");
        Ok(())
    }

    /// Confirms that multiple lines of output are captured in their original form.
    #[traced_test]
    fn test_capture_stdout_multiple_lines() -> Result<(), Box<dyn Error>> {
        let captured = capture_stdout(|| {
            println!("Line one");
            println!("Line two");
            println!("Line three");
        })?;
        let expected = "Line one\nLine two\nLine three\n";
        assert_eq!(captured, expected);
        Ok(())
    }

    /// Checks that very large output is captured fully without truncation.
    #[traced_test]
    fn test_capture_stdout_large_output() -> Result<(), Box<dyn Error>> {
        // Generate a large string of repeated characters.
        let large_data = "a".repeat(8192);
        let captured = capture_stdout(|| {
            print!("{}", large_data);
        })?;
        assert_eq!(captured.len(), 8192);
        assert!(captured.chars().all(|c| c == 'a'));
        Ok(())
    }

    /// Ensures that non-UTF8 byte sequences are captured and converted via `from_utf8_lossy`.
    #[traced_test]
    fn test_capture_stdout_non_utf8() -> Result<(), Box<dyn Error>> {
        // Some arbitrary bytes that are not valid UTF-8 for certain code points.
        let bad_bytes = [0x66, 0x6F, 0x80, 0xFE, 0xFF];
        let captured = capture_stdout(|| {
            // Write raw bytes to stdout. This requires a lower-level write.
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            // In real usage, handle errors appropriately; here we expect.
            handle.write_all(&bad_bytes).expect("expected successful write_all");
        })?;

        // The output won't match the original bytes exactly due to UTF-8 replacement,
        // but it should contain the valid ones and replace invalid ones.
        // "fo" is valid ASCII, so they remain unchanged; the other bytes become replacement chars.
        assert!(captured.starts_with("fo"));
        assert!(captured.contains('\u{FFFD}'));
        Ok(())
    }

    /// Verifies that stdout is restored even if the closure panics mid-execution.
    ///
    /// We cannot validate partial output in a normal test because the panic
    /// interrupts printing, but at minimum we ensure that the function
    /// does not leave stdout unusable after returning.
    #[traced_test]
    fn test_capture_stdout_closure_panics() {
        use std::panic;

        let result = panic::catch_unwind(|| {
            let _ = capture_stdout(|| {
                println!("This should partially print before panicking.");
                panic!("Simulated failure");
            });
        });

        // We expect the closure to panic.
        assert!(result.is_err());
        // If stdout wasn't restored, subsequent prints in this test could fail,
        // but we rely on normal Cargo test execution to confirm restoration.
        println!("Stdout should still work after the panic.");
    }
}
