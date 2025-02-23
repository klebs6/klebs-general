// ---------------- [ File: src/ensure_git_clean.rs ]
crate::ix!();

#[async_trait]
pub trait EnsureGitClean {
    type Error;
    async fn ensure_git_clean(&self) -> Result<(), Self::Error>;
}

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

#[async_trait]
impl<P,H> EnsureGitClean for Workspace<P,H>
where
    // your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Error = GitError;

    /// Checks that the Git working directory is clean:
    ///  - If `git status --porcelain` returns any output, we fail.
    ///  - If there's no .git folder or `git` isn't installed, this will also error out.
    async fn ensure_git_clean(&self) -> Result<(), Self::Error> {
        // Run `git status --porcelain`; if it returns any output, that means dirty/untracked changes
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(self.as_ref()) // important: run in workspace directory
            .output()
            .await
            .map_err(|e|
                GitError::IoError { 
                    io: Arc::new(e),
                    context: format!("could not run git status --porcelain in current directory: {:?}", self.as_ref()),
                }
            )?;

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
mod test_ensure_git_clean_for_workspace {
    use super::*;

    // We'll define a minimal "MockWorkspace" or actually a real `Workspace<_,_>` if you can:
    // For demonstration, let's say your real code can do:
    //   let ws = Workspace::new(&some_path).await?
    // and `ws.ensure_git_clean().await?` calls the trait method.
    //
    // If you need a minimal test-only struct that implements `EnsureGitClean`, we can do that.

    /// Helper to run a shell command in the given directory (blocking). 
    /// For a purely async test, you might do tokio::process::Command -> await.
    async fn run_in_dir(dir: &std::path::Path, cmd: &str, args: &[&str]) -> Result<(), String> {
        let mut command = Command::new(cmd);
        command.args(args).current_dir(dir);
        let output = command.output().await.map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Command {} {:?} failed: {}", cmd, args, stderr));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_clean_repo_succeeds() {
        // 1) Create a temp directory
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path();

        // 2) Initialize a git repo
        //    We'll skip if `git` is not installed, or just let it fail with "Failed to spawn"
        run_in_dir(path, "git", &["init", "."]).await.expect("git init should succeed");
        // 3) Create a file
        let file_path = path.join("hello.txt");
        tokio::fs::write(&file_path, b"hello").await.expect("write file");
        // 4) git add + commit
        run_in_dir(path, "git", &["add", "."]).await.expect("git add .");
        run_in_dir(path, "git", &["commit", "-m", "Initial commit"]).await.expect("git commit");

        // 5) Build or mock your workspace that references `path`.
        //    For demonstration, let's define a small struct implementing `AsRef<Path>`.
        let ws = MockWorkspace { root: path.to_path_buf() };

        // 6) Call ensure_git_clean
        let result = ws.ensure_git_clean().await;
        assert!(result.is_ok(), "A fully committed repo is clean => Ok(())");
    }

    #[tokio::test]
    async fn test_dirty_repo_fails() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path();

        run_in_dir(path, "git", &["init", "."]).await.expect("git init");
        // create a file
        let file_path = path.join("hello.txt");
        tokio::fs::write(&file_path, b"hello").await.expect("write file");
        // We do NOT commit it => working dir is dirty

        let ws = MockWorkspace { root: path.to_path_buf() };

        let result = ws.ensure_git_clean().await;
        match result {
            Err(GitError::WorkingDirectoryIsNotCleanAborting) => {
                // expected
            }
            other => panic!("Expected WorkingDirectoryIsNotCleanAborting, got {:?}", other),
        }
    }

    /// If there's no .git folder or `git` isn't installed, we likely get an IoError or
    /// GitError::FailedToRunGitStatusMakeSureGitIsInstalled, depending on the error message.
    /// We'll demonstrate a scenario with no .git:
    #[tokio::test]
    async fn test_not_git_repo_errors() {
        let tmp_dir = tempdir().expect("failed to create temp dir");
        let path = tmp_dir.path();

        let ws = MockWorkspace { root: path.to_path_buf() };

        let result = ws.ensure_git_clean().await;
        match result {
            Err(GitError::IoError{..}) 
            | Err(GitError::FailedToRunGitStatusMakeSureGitIsInstalled) => {
                // Either error is plausible if there's no .git or no git command
            }
            other => panic!("Expected IoError or FailedToRunGitStatus..., got {:?}", other),
        }
    }

    // A minimal workspace that implements `AsRef<Path>` and references the same `ensure_git_clean`
    // logic you posted.  If you have a real `Workspace<P,H>`, just use that.
    #[derive(Debug)]
    struct MockWorkspace {
        root: PathBuf,
    }

    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.root
        }
    }

    #[async_trait]
    impl EnsureGitClean for MockWorkspace {
        type Error = GitError;

        async fn ensure_git_clean(&self) -> Result<(), Self::Error> {
            // replicate your code:
            let output = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(self.as_ref())
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

    /// Creates a new temp directory, writes a minimal Cargo.toml, and initializes a Git repo.
    async fn setup_git_repo_for_crate() -> (TempCratePath, PathBuf) {
        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = tmp_dir.path().to_path_buf();

        // Write Cargo.toml
        write_minimal_cargo_toml(&repo_path).await;

        // Initialize Git repository
        run_in_dir("git", &["init"], &repo_path)
            .await
            .expect("Failed to init Git repository");

        // Add and commit
        run_in_dir("git", &["add", "."], &repo_path)
            .await
            .expect("Failed to git add .");
        run_in_dir("git", &["commit", "-m", "Initial commit"], &repo_path)
            .await
            .expect("Failed to git commit");

        (TempCratePath(repo_path.clone()), repo_path)
    }

    /// Test 1: We verify that `ensure_git_clean` succeeds on a pristine repo (no changes).
    #[tokio::test]
    async fn test_ensure_git_clean_pristine_repo() {
        let (temp_crate_path, _repo_path) = setup_git_repo_for_crate().await;
        // Build the CrateHandle
        let handle = CrateHandle::new(&temp_crate_path)
            .await
            .expect("Failed to create CrateHandle");

        // Should succeed because there are no uncommitted changes
        handle.ensure_git_clean().await.expect("Expected clean repo");
    }

    /// Test 2: We verify that `ensure_git_clean` fails if there's an untracked file in the repo.
    #[tokio::test]
    async fn test_ensure_git_clean_untracked_changes() {
        let (temp_crate_path, repo_path) = setup_git_repo_for_crate().await;
        let handle = CrateHandle::new(&temp_crate_path)
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
    }

    /// Test 3: We verify that `ensure_git_clean` fails if there's a modified file (tracked, but uncommitted changes).
    #[tokio::test]
    async fn test_ensure_git_clean_modified_file() {
        let (temp_crate_path, repo_path) = setup_git_repo_for_crate().await;
        let handle = CrateHandle::new(&temp_crate_path)
            .await
            .expect("Failed to create CrateHandle");

        // We'll modify the existing Cargo.toml
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

        // We do not commit this change => it's a local modification
        let result = handle.ensure_git_clean().await;
        assert!(
            matches!(result, Err(GitError::WorkingDirectoryIsNotCleanAborting)),
            "Expected WorkingDirectoryIsNotCleanAborting, got: {result:?}"
        );
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
}
