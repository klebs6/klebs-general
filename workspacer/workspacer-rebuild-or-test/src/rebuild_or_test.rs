// ---------------- [ File: src/rebuild_or_test.rs ]
crate::ix!();

#[async_trait]
pub trait RebuildOrTest {

    type Error;

    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> RebuildOrTest for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error> {

        let workspace_path = self.as_ref();

        info!("Running cargo build...");

        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build").current_dir(&workspace_path);

        let output = runner.run_command(build_cmd).await??;

        if !output.status.success() {
            error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(WorkspaceError::from(BuildError::BuildFailed {
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }));
        }

        info!("Rebuild succeeded, running tests...");

        let mut test_cmd = Command::new("cargo");
        test_cmd.arg("test").current_dir(&workspace_path);

        let test_output = runner.run_command(test_cmd).await??;

        if !test_output.status.success() {
            let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
            let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());

            error!("Tests failed: {:#?}", stderr);
            return Err(WorkspaceError::from(TestFailure::UnknownError { stdout, stderr }));
        }

        info!("Tests passed successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod test_rebuild_or_test_real {
    use super::*;
    use workspacer_3p::tokio::process::Command;
    use workspacer_3p::tokio;
    use tempfile::tempdir;
    use std::path::PathBuf;

    // Suppose you have `Workspace<P,H>` in your real code. 
    // For demonstration, we'll define a minimal mock that still calls your real rebuild_or_test logic:
    // Or you can skip a mock if you have a constructor for your real Workspace on disk.
    #[derive(Debug)]
    struct MockWorkspace {
        path: PathBuf,
    }

    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.path
        }
    }

    // We'll define or use your real runner if you have a `DefaultCommandRunner`.
    // For demonstration, let's assume we have it:
    use crate::{DefaultCommandRunner, RebuildOrTest, CommandRunner, WorkspaceError, TestFailure, BuildError};

    #[async_trait]
    impl RebuildOrTest for MockWorkspace {
        type Error = WorkspaceError;

        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error> {
            // replicate or directly use your posted code:
            // ...
            let workspace_path = self.as_ref();

            info!("Running cargo build...");
            let mut build_cmd = Command::new("cargo");
            build_cmd.arg("build").current_dir(&workspace_path);

            let output = runner.run_command(build_cmd).await??;
            if !output.status.success() {
                error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
                return Err(WorkspaceError::from(BuildError::BuildFailed {
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                }));
            }

            info!("Rebuild succeeded, running tests...");
            let mut test_cmd = Command::new("cargo");
            test_cmd.arg("test").current_dir(&workspace_path);

            let test_output = runner.run_command(test_cmd).await??;
            if !test_output.status.success() {
                let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
                let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());
                error!("Tests failed: {:#?}", stderr);
                return Err(WorkspaceError::from(TestFailure::UnknownError { stdout, stderr }));
            }

            info!("Tests passed successfully.");
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_rebuild_or_test_succeeds() {
        // 1) Set up a minimal Cargo project in a temp dir
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path();

        // Initialize a cargo project:
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("Failed to run cargo init");
        assert!(init_status.status.success(), "cargo init must succeed");

        // Optionally put some passing test in src/main.rs:
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main(){ println!("Hello"); }
            #[test] fn test_ok(){ assert_eq!(2+2,4); }
        "#;
        tokio::fs::write(&main_rs, code).await.expect("write main.rs");

        // 2) Construct our workspace
        let ws = MockWorkspace { path: path.to_path_buf() };

        // 3) Call rebuild_or_test using the default runner
        let runner = DefaultCommandRunner;
        let result = ws.rebuild_or_test(&runner).await;

        // 4) Confirm success
        assert!(result.is_ok(), "Should build and test successfully");
    }

    /// If the build fails, we expect `BuildFailed`.
    #[tokio::test]
    async fn test_rebuild_or_test_build_fails() {
        let tmp_dir = tempdir().expect("tempdir");
        let path = tmp_dir.path();
        // cargo init
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("cargo init");
        assert!(init_status.status.success());

        // Insert code that won't compile
        let main_rs = path.join("src").join("main.rs");
        let broken_code = b"fn main() { let x:doesnotexist = 42; }";
        tokio::fs::write(&main_rs, broken_code).await.expect("write broken code");

        let ws = MockWorkspace { path: path.to_path_buf() };
        let runner = DefaultCommandRunner;
        let result = ws.rebuild_or_test(&runner).await;
        match result {
            Err(WorkspaceError::BuildError(BuildError::BuildFailed{stderr})) => {
                println!("Build error: {}", stderr);
            }
            Ok(_) => panic!("Expected build to fail"),
            other => panic!("Expected BuildFailed, got {:?}", other),
        }
    }

    /// If the build succeeds but tests fail, we expect a `TestFailure`.
    #[tokio::test]
    async fn test_rebuild_or_test_test_fails() {
        let tmp_dir = tempdir().expect("tempdir");
        let path = tmp_dir.path();
        // cargo init
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--bin")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("cargo init");
        assert!(init_status.status.success());

        // Insert code with a failing test
        let main_rs = path.join("src").join("main.rs");
        let code = r#"
            fn main(){ println!("Hello"); }
            #[test] fn test_fail(){ assert_eq!(1+1,3); }
        "#;
        tokio::fs::write(&main_rs, code).await.expect("write main.rs");

        let ws = MockWorkspace { path: path.to_path_buf() };
        let runner = DefaultCommandRunner;
        let result = ws.rebuild_or_test(&runner).await;
        match result {
            Err(WorkspaceError::TestFailure(TestFailure::UnknownError{stdout, stderr})) => {
                println!("Test failure: stdout={:?}, stderr={:?}", stdout, stderr);
            }
            Ok(_) => panic!("Expected tests to fail"),
            other => panic!("Expected UnknownError, got {:?}", other),
        }
    }
}
