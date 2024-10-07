crate::ix!();

pub const DEFAULT_MANIFEST_PATH: &'static str = "manifest/default.json"; 

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    repos: Vec<Repo>,
}

impl Manifest {

    pub fn load(file_path: &str) -> Result<Manifest, std::io::Error> {
        let file_content = fs::read_to_string(file_path)?;
        let manifest: Manifest = serde_json::from_str(&file_content)?;
        Ok(manifest)
    }

    pub fn n_repos(&self) -> usize {
        self.repos.len()
    }

    pub fn repo(&self, idx: usize) -> Option<&Repo> {
        self.repos.get(idx)
    }
}

impl ProcessRepos for Manifest {
    fn process_repos(&self, operation: RepoOperation) -> Result<(), git2::Error> {
        let mut clean_repos = Vec::new();
        let mut warning_repos = Vec::new();

        self.repos.iter().for_each(|repo| {
            let repo_name = repo.name().to_string();
            let status_result = match operation {
                RepoOperation::Status => repo.check_status(),
                RepoOperation::Sync => repo.sync_repo(),
            };

            match status_result {
                Ok(status) => {
                    if status.is_clean && status.is_pushed {
                        clean_repos.push(repo_name);
                    } else {
                        warning_repos.push((repo_name, status));
                    }
                }
                Err(e) => {
                    error!("Error processing repo {}: {}", repo_name, e);
                }
            }
        });

        // Log clean repos
        if !clean_repos.is_empty() {
            info!("The following repositories are clean and pushed upstream:\n");
            for repo in &clean_repos {
                info!("{}", repo);
            }
            info!("All clean repositories processed successfully.\n");
        }

        // Log repos with warnings
        if !warning_repos.is_empty() {
            warn!("The following repositories have issues that need to be addressed:\n");
            for (repo_name, status) in &warning_repos {
                let mut dirty_loop = false;
                if !status.unstaged_changes.is_empty() {
                    dirty_loop = true;
                    warn!("{} has unstaged changes in the following files:", repo_name);
                    for file in &status.unstaged_changes {
                        warn!("- {}", file);
                    }
                }

                if !status.untracked_files.is_empty() {
                    dirty_loop = true;
                    warn!("{} has untracked files or directories:", repo_name);
                    for file in &status.untracked_files {
                        warn!("- {}", file);
                    }
                }

                if !status.staged_changes.is_empty() {
                    dirty_loop = true;
                    warn!("{} has staged changes ready for commit:", repo_name);
                    for file in &status.staged_changes {
                        warn!("- {}", file);
                    }
                }

                if status.unpushed_commits {
                    dirty_loop = true;
                    warn!("{} has local commits that need to be pushed.", repo_name);
                }

                if dirty_loop {
                    println!("");
                }
            }
        }

        Ok(())
    }
}
