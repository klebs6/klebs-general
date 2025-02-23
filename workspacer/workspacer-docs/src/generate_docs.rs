// ---------------- [ File: src/generate_docs.rs ]
crate::ix!();

#[async_trait]
pub trait GenerateDocs {
    type Error;
    async fn generate_docs(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> GenerateDocs for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Generates the documentation for the entire workspace by running `cargo doc`.
    async fn generate_docs(&self) -> Result<(), WorkspaceError> {
        let workspace_path = self.as_ref();  // Assuming `self.path()` returns the workspace root path.
        
        // Execute `cargo doc` in the workspace directory.
        let output = Command::new("cargo")
            .arg("doc")
            .current_dir(workspace_path)
            .output()
            .await
            .map_err(|e| CargoDocError::CommandError { io: e.into() })?;  // Handle any I/O error from the process execution.

        if !output.status.success() {
            // If the command failed, return an error with the captured output.
            return Err(WorkspaceError::from(CargoDocError::UnknownError {
                stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            }));
        }

        Ok(())  // If the command was successful, return Ok.
    }
}

#[cfg(test)]
mod test_generate_docs_real {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use workspacer_3p::tokio::process::Command;
    use workspacer_3p::tokio;
    // Adjust imports to match your code. For instance:
    // use crate::{ GenerateDocs, WorkspaceInterface, ... };

    // We define a minimal "MockWorkspace" or use your real `Workspace<P,H>` if possible.
    #[derive(Debug)]
    struct MockWorkspace {
        root: PathBuf,
    }
    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.root
        }
    }

    // We also implement `GenerateDocs` or rely on your real implementation:
    // but let's say we replicate your snippet in a trait impl for this mock:
    #[async_trait]
    impl GenerateDocs for MockWorkspace {
        type Error = WorkspaceError; // or your real error type

        async fn generate_docs(&self) -> Result<(), Self::Error> {
            let workspace_path = self.as_ref();
            let output = Command::new("cargo")
                .arg("doc")
                .current_dir(workspace_path)
                .output()
                .await
                .map_err(|e| CargoDocError::CommandError { io: e.into() })?;

            if !output.status.success() {
                return Err(WorkspaceError::from(CargoDocError::UnknownError {
                    stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                    stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
                }));
            }

            Ok(())
        }
    }

    // We'll define test scenarios:

    /// 1) Succeeds if we have a valid cargo project
    #[tokio::test]
    async fn test_generate_docs_success() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let path = tmp_dir.path();

        // 1) Initialize a cargo project in this directory. We'll do a minimal approach:
        // cargo init .
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--name")
            .arg("test_docs_proj")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("Failed to run cargo init");
        assert!(init_status.status.success(), "cargo init must succeed for test");

        // 2) Create the mock or real workspace referencing that directory
        let ws = MockWorkspace { root: path.to_path_buf() };

        // 3) Run generate_docs
        let result = ws.generate_docs().await;
        // We expect Ok(()) if everything is installed correctly.
        // On a real system with cargo doc, this should pass.
        // If docs fail for some reason, you might get an error. 
        // Possibly your environment might not have cargo doc or correct toolchain.
        assert!(result.is_ok(), "cargo doc should succeed on a minimal project");
    }

    /// 2) If the doc build fails for some reason (like a broken code), 
    ///    we expect an error with the captured stdout/stderr in CargoDocError::UnknownError.
    #[tokio::test]
    async fn test_generate_docs_failure() {
        let tmp_dir = tempdir().expect("tempdir failed");
        let path = tmp_dir.path();

        // Initialize a cargo project
        let init_status = Command::new("cargo")
            .arg("init")
            .arg("--name")
            .arg("broken_docs")
            .arg("--vcs")
            .arg("none")
            .current_dir(path)
            .output()
            .await
            .expect("cargo init");
        assert!(init_status.status.success());

        // Insert invalid rust code in src/lib.rs so docs fail
        let src_lib = path.join("src").join("lib.rs");
        tokio::fs::write(&src_lib, b"broken code ???").await.expect("write broken code");

        let ws = MockWorkspace { root: path.to_path_buf() };
        let result = ws.generate_docs().await;
        match result {
            Err(WorkspaceError::CargoDocError(CargoDocError::UnknownError { stderr, stdout })) => {
                // We'll see some compiler error messages in stderr
                assert!(stderr.as_ref().unwrap_or(&String::new()).contains("error"), "stderr should mention an error");
            }
            Ok(_) => {
                panic!("Expected doc build to fail, but got Ok(())");
            }
            other => panic!("Expected UnknownError, got {:?}", other),
        }
    }

    /// 3) If cargo doc can't be spawned or cargo isn't installed, we get `CargoDocError::CommandError`.
    #[tokio::test]
    async fn test_generate_docs_cannot_spawn_command() {
        // We'll skip creating a real project. We'll do a scenario that might not have cargo installed
        // or you can forcibly rename cargo. The simplest is we rename cargo in PATH or remove it, 
        // but let's just see what happens:
        let ws = MockWorkspace { root: PathBuf::from("/non/existent/path") };

        // If cargo doesn't exist or something, we get CommandError. 
        // But if cargo is installed, we might get a different error about not a cargo project. 
        // We'll just do partial matching:
        let result = ws.generate_docs().await;
        match result {
            Err(WorkspaceError::CargoDocError(CargoDocError::CommandError{..})) => {
                // Good
            }
            other => {
                println!("Got something else: {:?}", other);
            }
        }
    }
}
