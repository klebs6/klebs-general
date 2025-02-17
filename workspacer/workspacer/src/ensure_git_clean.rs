crate::ix!();

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
