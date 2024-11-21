crate::ix!();

pub struct RepoStatus {
    pub is_clean:         bool,        // Whether the working tree is clean
    pub is_pushed:        bool,        // Whether all commits are pushed upstream
    pub unstaged_changes: Vec<String>, // List of unstaged files
    pub untracked_files:  Vec<String>, // List of untracked files
    pub staged_changes:   Vec<String>, // List of staged but uncommitted files
    pub unpushed_commits: bool,        // Whether there are local commits not pushed
}

impl Repo {

    pub fn sync_repo(&self) -> Result<RepoStatus, git2::Error> {
        todo!("need to test `rck sync` -- TODO");
        self.git_sync()?;
        self.check_status()
    }

    pub fn check_status(&self) -> Result<RepoStatus, git2::Error> {
        let repo_status      = self.git_status_clean()?; // Collect unstaged changes and untracked files
        let is_pushed        = self.is_pushed_upstream()?;
        let unpushed_commits = !is_pushed; // Check for unpushed commits

        Ok(RepoStatus {
            is_clean: repo_status.is_clean,
            is_pushed,
            unstaged_changes: repo_status.unstaged_changes,
            untracked_files: repo_status.untracked_files, // Properly propagate untracked files
            staged_changes: repo_status.staged_changes,   // Properly propagate staged changes
            unpushed_commits,
        })
    }
}
