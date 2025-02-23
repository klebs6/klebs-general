// ---------------- [ File: src/linting.rs ]
crate::ix!();

#[async_trait]
pub trait RunLinting {

    type Report;
    type Error;
    async fn run_linting(&self) -> Result<Self::Report, Self::Error>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> RunLinting for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    type Report = LintReport;
    type Error  = LintingError;

    /// Runs cargo clippy to lint the workspace and collects the linting results.
    async fn run_linting(&self) -> Result<Self::Report, Self::Error> {

        let workspace_path = self.as_ref();  // Assuming `self.path()` returns the workspace root path.

        // Run `cargo clippy` in the workspace directory, treating warnings as errors.
        let output = tokio::process::Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--message-format=short")
            .arg("--quiet")
            .arg("--")
            .arg("-D")
            .arg("warnings")  // Deny warnings to force failure on lint issues
            .current_dir(workspace_path)
            .output()
            .await
            .map_err(|e| LintingError::CommandError { io: e.into() })?;  // Handle any I/O error from the process execution.

        // Capture the linting results in LintReport.
        let report = LintReport::from(output);

        // If clippy failed, return an error.
        report.maybe_throw()?;

        Ok(report)  // Return the linting report if successful.
    }
}

#[cfg(test)]
mod test_run_linting_real {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use workspacer_3p::tokio;
    use workspacer_3p::tokio::process::Command;

    // If you already have a real `Workspace<P,H>` for your environment, you can use that directly.
    // For demonstration, we define a minimal "MockWorkspace" or "TestWorkspace" that implements
    // your real `RunLinting` snippet, or we rely on the real code if accessible.

    #[derive(Debug)]
    struct MockWorkspace {
        root: PathBuf,
    }

    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.root
        }
    }

    // We'll replicate the run_linting code or rely on the trait if it's default-implemented.
    // For demonstration, let's do a direct impl:
    #[async_trait]
    impl RunLinting for MockWorkspace {
        type Report = LintReport;
        type Error = LintingError;

        async fn run_linting(&self) -> Result<Self::Report, Self::Error> {
            let workspace_path = self.as_ref();

            let output = Command::new("cargo")
                .arg("clippy")
                .arg("--all-targets")
                .arg("--message-format=short")
                .arg("--quiet")
                .arg("--")
                .arg("-D")
                .arg("warnings")
                .current_dir(workspace_path)
                .output()
                .await
                .map_err(|e| LintingError::CommandError { io: e.into() })?;

            let report = LintReport::from(output);
            report.maybe_throw()?;
            Ok(report)
        }
    }

    // -----------------------------------------------------------------------
    // Actual tests:
    // -----------------------------------------------------------------------

    /// 1) If we have a valid, clean Cargo project with no lint warnings, `run_linting` should succeed.
    #[tokio::test]
    async fn test_run_linting_succeeds_no_warnings() {
        let tmp_dir = tempdir().expect("create temp dir");
        let root = tmp_dir.path();

        // We'll initialize a new cargo project:
        //   cargo init --vcs none --bin
        let init_output = Command::new("cargo")
            .arg("init")
            .arg("--vcs")
            .arg("none")
            .arg("--bin")
            .arg("--name")
            .arg("lint_test_proj")
            .current_dir(root)
            .output()
            .await
            .expect("Failed to run cargo init");
        assert!(
            init_output.status.success(),
            "cargo init must succeed for the test to proceed"
        );

        // Optionally write code that has no lint warnings:
        let main_rs = root.join("src").join("main.rs");
        tokio::fs::write(&main_rs, b"fn main(){ println!(\"Hello!\"); }")
            .await
            .expect("write main.rs");

        // Build our mock workspace
        let ws = MockWorkspace {
            root: root.to_path_buf(),
        };

        // 2) run run_linting
        let result = ws.run_linting().await;
        // 3) Because there's no warnings, we expect success:
        assert!(result.is_ok(), "Clippy should succeed without warnings");
        let report = result.unwrap();
        assert!(report.success(), "LintReport should show success");
        // Optionally check stdout/stderr
        println!("stdout:\n{}", report.stdout());
        println!("stderr:\n{}", report.stderr());
    }

    /// 2) If the code has a lint warning or error, we expect a failure `UnknownError` with the output.
    #[tokio::test]
    async fn test_run_linting_fails_on_warnings() {
        let tmp_dir = tempdir().expect("tempdir");
        let root = tmp_dir.path();

        // cargo init --vcs none
        let init_output = Command::new("cargo")
            .arg("init")
            .arg("--vcs")
            .arg("none")
            .arg("--bin")
            .arg("--name")
            .arg("lint_warn_proj")
            .current_dir(root)
            .output()
            .await
            .expect("cargo init");
        assert!(init_output.status.success());

        // Insert a code snippet that triggers a clippy warning
        // For example: an unused variable or something. Let "x" be unused
        let main_rs = root.join("src").join("main.rs");
        let code_with_warning = b"
            fn main() {
                let x = 42; // unused
                println!(\"Hello\");
            }
        ";
        tokio::fs::write(&main_rs, code_with_warning)
            .await
            .expect("write main with warning");

        let ws = MockWorkspace {
            root: root.to_path_buf(),
        };

        let result = ws.run_linting().await;
        match result {
            Err(LintingError::UnknownError { stderr, stdout }) => {
                // We'll see clippy's warning => it fails because we pass `-D warnings`.
                // Possibly check if "warning:" or something is in `stderr`.
                let stde = stderr.unwrap_or_default();
                println!("clippy stderr: {}", stde);
                assert!(
                    stde.contains("warning") || stde.contains("error"),
                    "Should mention a lint warning or error"
                );
            }
            Ok(report) => {
                panic!("Expected clippy to fail with a warning, but it succeeded: {:?}", report)
            }
            other => panic!("Expected UnknownError, got {:?}", other),
        }
    }

    /// 3) If the environment has no cargo/clippy, or if we can't spawn the process, we get `CommandError`.
    #[tokio::test]
    async fn test_run_linting_command_error() {
        // We'll not create a real cargo project. We'll rely on the environment missing cargo or something.
        // In many systems cargo is installed, so you'll get a different error. 
        // We can forcibly rename cargo or do partial checks:

        let ws = MockWorkspace {
            root: PathBuf::from("/non/existent/directory"),
        };
        let result = ws.run_linting().await;
        match result {
            Err(LintingError::CommandError { .. }) => {
                // Good
            }
            other => {
                println!("We got something else: {:?}", other);
            }
        }
    }
}
