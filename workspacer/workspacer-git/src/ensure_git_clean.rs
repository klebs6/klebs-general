// ---------------- [ File: src/ensure_git_clean.rs ]
crate::ix!();

#[async_trait]
pub trait EnsureGitClean {
    type Error;
    async fn ensure_git_clean(&self) -> Result<(), Self::Error>;
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
