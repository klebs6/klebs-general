// ---------------- [ File: lightweight-command-runner/src/command_runner.rs ]
crate::ix!();

pub trait CommandRunner: Send + Sync {

    fn run_command(&self, cmd: tokio::process::Command) 
        -> tokio::task::JoinHandle<Result<std::process::Output, io::Error>>;
}

pub struct DefaultCommandRunner;

impl CommandRunner for DefaultCommandRunner {

    fn run_command(&self, cmd: tokio::process::Command) 
        -> tokio::task::JoinHandle<Result<std::process::Output, io::Error>> 
    {
        tokio::spawn(async move {
            let mut cmd = cmd;
            cmd.output().await
        })
    }
}

#[cfg(test)]
mod test_command_runner {
    use super::*;
    use std::process::Output;
    use tokio::process::Command;
    use tokio::runtime::Runtime;

    // We'll write multiple tests covering different scenarios:
    // 1) A successful command (like `echo "Hello"`) that should return exit code 0.
    // 2) A failing command (like `thisShouldNotExist`) that fails to launch or returns a non-zero exit code.
    // 3) A test specifically for verifying stdout/stderr content, if feasible.
    // 4) Tests for `make_exit_status` using Unix or Windows raw codes, ensuring they produce the correct exit status.

    /// Creates a new `DefaultCommandRunner` for testing.
    fn create_command_runner() -> DefaultCommandRunner {
        DefaultCommandRunner
    }

    /// Test that a simple command completes successfully with exit code 0.
    /// We'll attempt a cross-platform approach using "echo" to print something.
    #[test]
    fn test_run_command_successfully() {
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let runner = create_command_runner();

            let mut cmd = if cfg!(windows) {
                let mut c = Command::new("cmd");
                c.arg("/C").arg("echo hello"); 
                c
            } else {
                let mut c = Command::new("echo");
                c.arg("hello");
                c
            };

            let handle = runner.run_command(cmd);
            let output_result = handle.await.expect("JoinHandle panicked");
            assert!(
                output_result.is_ok(),
                "Expected successful result from echo command"
            );
            let output = output_result.unwrap();
            assert!(
                output.status.success(),
                "Expected exit code 0 from echo command"
            );
        });
    }

    /// Test that running a non-existent command produces an error, or at least a non-zero exit code.
    #[test]
    fn test_run_command_non_existent() {
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let runner = create_command_runner();

            // We'll try a made-up command name that hopefully doesn't exist
            let cmd = if cfg!(windows) {
                // Windows might say "not recognized as an internal or external command"
                Command::new("thisCommandDefinitelyShouldNotExistOnWindows")
            } else {
                // Linux or Mac will typically say "No such file or directory"
                Command::new("thisCommandDefinitelyShouldNotExistOnUnix")
            };

            let handle = runner.run_command(cmd);
            let output_result = handle.await.expect("JoinHandle panicked");
            assert!(output_result.is_err() || !output_result.as_ref().unwrap().status.success(),
                "Expected an error or a failing exit code for non-existent command"
            );
        });
    }

    /// Test that we can capture stdout from a command that writes to stdout.
    #[test]
    fn test_run_command_stdout_capture() {
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let runner = create_command_runner();

            let mut cmd = if cfg!(windows) {
                let mut c = Command::new("cmd");
                c.arg("/C").arg("echo capture_this_stdout");
                c
            } else {
                let mut c = Command::new("echo");
                c.arg("capture_this_stdout");
                c
            };

            let handle = runner.run_command(cmd);
            let output_result = handle.await.expect("JoinHandle panicked");
            let output = match output_result {
                Ok(o) => o,
                Err(e) => panic!("Failed to run echo command: {e}"),
            };
            assert!(
                output.status.success(),
                "Expected exit code 0 from echo command"
            );

            // Convert stdout to string and see if it contains "capture_this_stdout"
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout_str.contains("capture_this_stdout"),
                "Expected stdout to contain 'capture_this_stdout', got: {stdout_str}"
            );
        });
    }

    /// Test that we can capture stderr from a command that writes to stderr.
    /// We'll intentionally run something that fails, so it prints to stderr.
    #[test]
    fn test_run_command_stderr_capture() {
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let runner = create_command_runner();

            // On Unix, `ls` a non-existent file typically prints to stderr.
            // On Windows, `dir` of a non-existent file also prints to stderr.
            let mut cmd = if cfg!(windows) {
                let mut c = Command::new("cmd");
                c.arg("/C").arg("dir thisDirectoryDoesNotExist");
                c
            } else {
                let mut c = Command::new("ls");
                c.arg("thisDirectoryDoesNotExist");
                c
            };

            let handle = runner.run_command(cmd);
            let output_result = handle.await.expect("JoinHandle panicked");
            let output = match output_result {
                Ok(o) => o,
                Err(e) => panic!("Failed to run 'ls/dir' command: {e}"),
            };

            assert!(
                !output.status.success(),
                "Expected a non-zero exit code for listing a non-existent directory"
            );
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr_str.is_empty(),
                "Expected non-empty stderr for listing a non-existent directory"
            );
        });
    }

    /// Test that we can construct and interpret exit status codes for Unix systems.
    #[cfg(unix)]
    #[test]
    fn test_make_exit_status_unix() {
        // raw=0 => success
        let status_success = make_exit_status(0);
        assert!(status_success.success());

        // raw=256 => means "exit code 1" on many Linux/BSD systems
        let status_error = make_exit_status(256);
        assert!(!status_error.success());
        assert_eq!(status_error.code(), Some(1), "Expected exit code 1");
    }
}
