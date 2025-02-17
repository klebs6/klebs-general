// ---------------- [ File: workspacer-crate/src/ensure_git_clean.rs ]
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
