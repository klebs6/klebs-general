crate::ix!();

impl GitStatusClean for Repo {
    fn git_status_clean(&self) -> Result<RepoStatus, git2::Error> {
        let repo_path = self.path();
        if repo_path.exists() && repo_path.is_dir() {
            let repository = Repository::open(repo_path.clone())?;
            let statuses = repository.statuses(None)?;

            let mut unstaged_changes = Vec::new();
            let mut untracked_dirs = HashSet::new();
            let mut untracked_files = HashSet::new();
            let mut staged_changes = Vec::new();

            for entry in statuses.iter() {
                let status = entry.status();
                let file_path = entry.path().unwrap_or("Unknown file").to_string();
                let path = Path::new(&file_path);

                // Handle unstaged changes (modified files only)
                if status.is_wt_modified() {
                    unstaged_changes.push(file_path.clone());
                }

                // Handle staged changes
                if status.is_index_new() || status.is_index_modified() {
                    staged_changes.push(file_path.clone());
                }

                // Handle untracked files, and group them by top-level directory
                if status.is_wt_new() {
                    if let Some(parent_dir) = path.parent() {
                        // Get the repository root
                        let repo_root = repository.workdir().unwrap_or(repo_path.as_path());

                        // Determine the path relative to the repository root
                        let relative_path = path.strip_prefix(repo_root).unwrap_or(path);

                        // Extract the top-level directory
                        if let Some(top_dir) = relative_path.iter().next() {
                            untracked_dirs.insert(top_dir.to_string_lossy().to_string());
                        } else {
                            untracked_files.insert(file_path);
                        }
                    } else {
                        untracked_files.insert(file_path);
                    }
                }
            }

            // Combine untracked directories and files, prioritizing directories
            let mut untracked_combined: Vec<String> = untracked_files.into_iter().collect();
            untracked_combined.extend(untracked_dirs.into_iter());
            untracked_combined.sort();

            let is_clean = unstaged_changes.is_empty() && untracked_combined.is_empty() && staged_changes.is_empty();
            let is_pushed = self.is_pushed_upstream()?;

            Ok(RepoStatus {
                is_clean,
                is_pushed,
                unstaged_changes,
                untracked_files: untracked_combined,
                staged_changes,
                unpushed_commits: !is_pushed,
            })
        } else {
            warn!("{} does not exist", repo_path.display());
            Ok(RepoStatus {
                is_clean: false,
                is_pushed: false,
                unstaged_changes: vec![],
                untracked_files: vec![],
                staged_changes: vec![],
                unpushed_commits: false,
            })
        }
    }
}

