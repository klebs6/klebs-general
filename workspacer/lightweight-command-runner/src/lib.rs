// ---------------- [ File: lightweight-command-runner/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{command_runner}
x!{exit_status}

#[cfg(test)]
mod test_default_command_runner {
    use super::*;
    use tokio::process::Command;

    /// Tests that the DefaultCommandRunner correctly returns the command's output.
    #[tokio::test]
    async fn should_run_echo_command_successfully() {
        let runner = DefaultCommandRunner;
        // Use a cross-platform echo command.
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "echo hello"]);
            c
        } else {
            let mut c = Command::new("echo");
            c.arg("hello");
            c
        };

        let output = runner.run_command(cmd).await;
        assert!(output.is_ok(), "Command should run successfully");

        let output = output.unwrap();
        let output = output.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"), "Output should contain 'hello'");
    }
}
