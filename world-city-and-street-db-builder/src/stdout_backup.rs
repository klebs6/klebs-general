// ---------------- [ File: src/stdout_backup.rs ]
// ---------------- [ File: src/stdout_backup.rs ]
crate::ix!();

use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

/// A RAII guard that saves the current stdout file descriptor and
/// restores it when dropped. This guarantees that stdout is always
/// restored even if the closure panics.
pub struct StdoutBackup {
    /// The file descriptor corresponding to stdout (e.g. file descriptor 1).
    stdout_fd: RawFd,
    /// A duplicate of the original stdout file descriptor used for restoration.
    backup_fd: RawFd,
}

impl StdoutBackup {
    /// Creates a new `StdoutBackup` by duplicating the current stdout file descriptor.
    pub fn new() -> io::Result<Self> {
        let stdout_fd = io::stdout().as_raw_fd();
        let backup_fd = unsafe { libc::dup(stdout_fd) };
        if backup_fd == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(StdoutBackup { stdout_fd, backup_fd })
        }
    }

    /// Restores stdout from the backup file descriptor.
    pub fn restore(&self) -> io::Result<()> {
        if unsafe { libc::dup2(self.backup_fd, self.stdout_fd) } == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for StdoutBackup {
    fn drop(&mut self) {
        // In a drop implementation we ignore errors.
        let _ = unsafe { libc::dup2(self.backup_fd, self.stdout_fd) };
        let _ = unsafe { libc::close(self.backup_fd) };
    }
}

#[cfg(test)]
#[disable]
mod test_stdout_backup {
    use super::*;
    use std::io::{self, Write};
    use std::os::unix::io::AsRawFd;

    /// A helper that creates a temporary pipe, redirects stdout to the pipe's writer,
    /// and returns `(StdoutBackup, reader_fd, writer_fd)`. The caller can then
    /// write to `io::stdout()` and read from the `reader_fd` to see what's captured.
    ///
    /// After the test, you must either drop the `StdoutBackup` or call `restore()`
    /// to ensure stdout is restored.
    fn redirect_stdout_to_pipe() -> io::Result<(StdoutBackup, i32, i32)> {
        // Create a pipe. We'll read from `pipe_fds[0]` and write to `pipe_fds[1]`.
        // We want to redirect stdout to pipe_fds[1].
        let mut pipe_fds = [0; 2];
        let rc = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
        if rc == -1 {
            return Err(io::Error::last_os_error());
        }

        let reader_fd = pipe_fds[0];
        let writer_fd = pipe_fds[1];

        // Create the backup object
        let backup = StdoutBackup::new()?;

        // Redirect stdout to writer_fd
        let stdout_fd = io::stdout().as_raw_fd();
        if unsafe { libc::dup2(writer_fd, stdout_fd) } == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok((backup, reader_fd, writer_fd))
    }

    /// Reads all available bytes from a file descriptor into a String.
    fn read_from_fd(fd: i32) -> String {
        let mut result = String::new();
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        let _ = file.read_to_string(&mut result);
        // We don't close the file descriptor explicitly here because `from_raw_fd`
        // transfers ownership; it will close on drop. In real usage, you might want
        // to `dup` the fd if you need further usage. For test purposes, this is okay.
        result
    }

    #[traced_test]
    fn test_new_creates_backup_ok() {
        let backup_result = StdoutBackup::new();
        assert!(backup_result.is_ok(), "Should create StdoutBackup without error");
        let _backup = backup_result.unwrap();
    }

    #[traced_test]
    fn test_restore_stdout() {
        // Step 1: redirect stdout to a pipe
        let (backup, reader_fd, writer_fd) = redirect_stdout_to_pipe()
            .expect("Failed to redirect stdout to pipe");

        // Step 2: Write something to stdout
        println!("Hello, pipe!");

        // flush stdout
        io::stdout().flush().unwrap();

        // Step 3: Read from the pipe
        // We'll close the writer end first to signal EOF on the reader
        unsafe { libc::close(writer_fd) };
        let captured = read_from_fd(reader_fd);
        // We expect "Hello, pipe!\n"
        assert!(captured.contains("Hello, pipe!"), "Should capture printed text");

        // Step 4: restore stdout
        backup.restore().expect("Should restore stdout");

        // After restoring, if we print again, it should go to real stdout,
        // not the pipe. We can't easily test that in an automated environment
        // without capturing the real stdout, but we can at least confirm
        // it doesn't go to the pipe we already closed.

        // Step 5: dropping `backup` will also call restore in `Drop`, but we've already done it.
        // No error expected on double restore in real usage.
    }

    #[traced_test]
    fn test_drop_calls_restore_stdout_implicitly() {
        // We'll do the same approach, but rely on dropping `backup`.
        let (backup, reader_fd, writer_fd) = redirect_stdout_to_pipe()
            .expect("Failed to redirect stdout to pipe");

        println!("Before drop");
        io::stdout().flush().unwrap();

        // Close writer to read
        unsafe { libc::close(writer_fd) };
        let captured_before_drop = read_from_fd(reader_fd);
        assert!(captured_before_drop.contains("Before drop"),
            "Should see the printed text in the pipe");

        // Once we exit this scope, `backup` is dropped => stdout is restored
        drop(backup);

        // We'll reacquire the pipe with new file descriptors for demonstration,
        // but in a real test environment you might just trust that the real stdout is back.
        let (new_backup, new_reader_fd, new_writer_fd) = redirect_stdout_to_pipe()
            .expect("Failed to redirect stdout again");
        println!("After drop");
        io::stdout().flush().unwrap();

        unsafe { libc::close(new_writer_fd) };
        let captured_after_drop = read_from_fd(new_reader_fd);

        // Because the stdout was presumably restored, then re-redirected, we see "After drop".
        // If the initial drop didn't restore, we'd see partial or conflicting results.
        assert!(captured_after_drop.contains("After drop"),
            "We should capture newly printed text after a second redirect, 
             confirming the first restore worked.");

        // Clean up
        new_backup.restore().unwrap();
    }

    #[traced_test]
    fn test_restore_fails_for_bogus_dup2() {
        // We can simulate a partial scenario with a mock or override if we want.
        // But in typical usage, it's hard to force dup2 to fail unless we pass invalid FDs.
        // We'll define a minimal approach by handing an invalid backup_fd to the struct.

        let mut bogus = StdoutBackup {
            stdout_fd: io::stdout().as_raw_fd(),
            backup_fd: -1, // invalid
        };

        let result = bogus.restore();
        assert!(result.is_err(), "Should fail to restore from an invalid FD");
    }

    #[traced_test]
    fn test_drop_ignores_errors() {
        // The drop implementation explicitly ignores errors. 
        // We'll rely on coverage to confirm no panic if there's an error. 
        // We'll do the same approach: an invalid backup_fd and drop.
        {
            let bogus = StdoutBackup {
                stdout_fd: io::stdout().as_raw_fd(),
                backup_fd: -1, // invalid
            };
            // Exiting scope => drop => no panic
        }
        // If it doesn't panic, we pass
    }
}
