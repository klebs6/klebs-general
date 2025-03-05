// ---------------- [ File: workspacer-git/src/ensure_git_clean_for_crate.rs ]
crate::ix!();

#[async_trait]
impl EnsureGitClean for CrateHandle {
    type Error = GitError;

    async fn ensure_git_clean(&self) -> Result<(), Self::Error> {
        info!("Ensuring Git is clean (single crate) at {:?}", self.as_ref());
        // If you want the same logic as workspace, you can copy-paste. 
        // Or `todo!()` if you prefer. 
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(self.as_ref())  // important: run in crate's directory
            .output()
            .await
            .map_err(|e| GitError::IoError { 
                io: Arc::new(e),
                context: format!("could not run git status --porcelain in current directory: {:?}", self.as_ref()),
            })?;

        if !output.status.success() {
            return Err(GitError::FailedToRunGitStatusMakeSureGitIsInstalled);
        }
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        if !stdout_str.trim().is_empty() {
            return Err(GitError::WorkingDirectoryIsNotCleanAborting);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_ensure_git_clean_for_crate_handle {
    use super::*;
    use std::process::Stdio;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;
    use tokio::fs::{File, create_dir_all};

    /// Minimal helper struct that implements `HasCargoTomlPathBuf` so we can create a `CrateHandle`.
    /// We'll initialize a dummy Cargo.toml in a git repo for testing `ensure_git_clean`.
    #[derive(Clone)]
    struct TempCratePath(PathBuf);

    impl AsRef<Path> for TempCratePath {
        fn as_ref(&self) -> &Path {
            self.0.as_ref()
        }
    }

    /// Writes a minimal Cargo.toml to the specified directory.
    async fn write_minimal_cargo_toml(dir: &Path) {
        let cargo_toml = r#"
            [package]
            name = "test_crate"
            version = "0.1.0"
            authors = ["Someone <someone@example.com>"]
            license = "MIT"
        "#;
        let file_path = dir.join("Cargo.toml");
        create_dir_all(dir).await.expect("Failed to create directories");
        let mut f = File::create(&file_path).await.expect("Failed to create Cargo.toml");
        f.write_all(cargo_toml.as_bytes())
            .await
            .expect("Failed to write Cargo.toml");
    }

    /// Helper to run a command in the given directory, returning Ok(()) if exit code == 0.
    /// For debugging test failures, we also capture stdout/stderr.
    async fn run_in_dir(cmd: &str, args: &[&str], dir: &Path) -> Result<(), String> {
        let mut command = Command::new(cmd);
        command.args(args).current_dir(dir).stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = command
            .output()
            .await
            .map_err(|e| format!("Failed to spawn '{cmd}': {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "'{cmd} {:?}' failed with code {:?}\nstdout:\n{}\nstderr:\n{}",
                args,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }

    /// Test 4: If `git` is not installed or `git status --porcelain` fails in some other way,
    /// we expect `Err(GitError::FailedToRunGitStatusMakeSureGitIsInstalled)` or an IO error.
    /// It's challenging to reliably force that scenario in normal environments. 
    /// We can at least demonstrate how we might do it by overriding PATH or something. 
    /// Here, we'll just show a test that might get ignored typically:
    #[tokio::test]
    #[ignore = "Requires mocking or a system without git installed to run meaningfully"]
    async fn test_ensure_git_clean_no_git_available() {
        let (temp_crate_path, repo_path) = setup_git_repo_for_crate().await;
        // If we alter environment PATH in such a way that 'git' can't be found,
        // we'd force an IO error. That is OS-specific. We'll skip actual implementation.
        let handle = CrateHandle::new(&temp_crate_path)
            .await
            .expect("Failed to create CrateHandle");

        // Hypothetically remove or rename 'git' from PATH, etc.
        // For demonstration only:
        unsafe { std::env::set_var("PATH", "") };

        let result = handle.ensure_git_clean().await;
        unsafe { std::env::remove_var("PATH") }; // restore if we want
        assert!(
            matches!(
                result,
                Err(GitError::IoError{..}) | Err(GitError::FailedToRunGitStatusMakeSureGitIsInstalled)
            ),
            "Expected IoError or FailedToRunGitStatusMakeSureGitIsInstalled, got: {result:?}"
        );
    }

    /// Returns the `TempDir` so it stays alive and a `PathBuf` to that directory.
    /// (We keep the `TempDir` in scope the entire time, ensuring the directory is not deleted.)
    async fn setup_git_repo_for_crate() -> (TempDir, PathBuf) {
        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = tmp_dir.path().to_path_buf();

        // Write Cargo.toml, init, commit, etc.
        write_minimal_cargo_toml(tmp_dir.path()).await;
        run_in_dir("git", &["init"], tmp_dir.path()).await.unwrap();
        run_in_dir("git", &["add", "."], tmp_dir.path()).await.unwrap();
        run_in_dir("git", &["commit", "-m", "Initial commit"], tmp_dir.path()).await.unwrap();

        // Return both the TempDir and the path. The caller must keep the TempDir in scope.
        (tmp_dir, repo_path)
    }

    #[tokio::test]
    async fn test_ensure_git_clean_pristine_repo() {
        // Keep the TempDir alive as long as we need the files
        let (temp_crate_dir, repo_path) = setup_git_repo_for_crate().await;

        // Now create the CrateHandle using the `repo_path` (which still exists!):
        let handle = CrateHandle::new(&repo_path)
            .await
            .expect("Failed to create CrateHandle");

        // This should succeed, because there are no uncommitted changes
        handle.ensure_git_clean().await.expect("Expected clean repo");

        // As soon as this function returns, `temp_crate_dir` drops and the directory is cleaned up.
    }

    // Similarly fix the other tests:
    #[tokio::test]
    async fn test_ensure_git_clean_untracked_changes() {
        let (temp_crate_dir, repo_path) = setup_git_repo_for_crate().await;
        let handle = CrateHandle::new(&repo_path)
            .await
            .expect("Failed to create CrateHandle");

        // Create an untracked file
        let untracked_file = repo_path.join("untracked_file.txt");
        {
            let mut f = File::create(&untracked_file)
                .await
                .expect("Failed to create untracked file");
            f.write_all(b"This is untracked content.")
                .await
                .expect("Failed to write untracked content");
        }

        // Now ensure_git_clean should fail
        let result = handle.ensure_git_clean().await;
        assert!(
            matches!(result, Err(GitError::WorkingDirectoryIsNotCleanAborting)),
            "Expected WorkingDirectoryIsNotCleanAborting, got: {result:?}"
        );
        // Directory is cleaned up only after exiting this function
    }

    #[tokio::test]
    async fn test_ensure_git_clean_modified_file() {
        let (temp_crate_dir, repo_path) = setup_git_repo_for_crate().await;
        let handle = CrateHandle::new(&repo_path)
            .await
            .expect("Failed to create CrateHandle");

        // Modify the existing Cargo.toml (tracked but uncommitted)
        let cargo_toml = repo_path.join("Cargo.toml");
        {
            let mut f = File::options()
                .append(true)
                .open(&cargo_toml)
                .await
                .expect("Failed to open Cargo.toml for appending");
            f.write_all(b"# Adding a new line to Cargo.toml\n")
                .await
                .expect("Failed to write to Cargo.toml");
        }

        let result = handle.ensure_git_clean().await;
        assert!(
            matches!(result, Err(GitError::WorkingDirectoryIsNotCleanAborting)),
            "Expected WorkingDirectoryIsNotCleanAborting, got: {result:?}"
        );
    }
}
