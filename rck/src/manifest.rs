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

        self.repos.par_iter().for_each(|repo| {
            info!("Processing repo: {}", repo.name());

            // Check status
            if operation == RepoOperation::Status || operation == RepoOperation::Sync {

                match repo.git_status_clean() {
                    Ok(clean) => {
                        if clean {
                            info!("{} is clean", repo.name());
                        } else {
                            warn!("{} has uncommitted changes", repo.name());
                        }
                    }
                    Err(e) => {
                        error!("Error checking status for {}: {}", repo.name(), e);
                    }
                }

                match repo.is_pushed_upstream() {
                    Ok(pushed) => {
                        if pushed {
                            info!("{} is pushed upstream", repo.name());
                        } else {
                            warn!("{} has local commits not pushed", repo.name());
                        }
                    }
                    Err(e) => {
                        error!("Error checking if pushed upstream for {}: {}", repo.name(), e);
                    }
                }
            }

            // Sync if operation is "sync"
            if operation == RepoOperation::Sync {
                if let Err(e) = repo.git_sync() {
                    error!("Error syncing repo {}: {}", repo.name(), e);
                }
            }
        });

        Ok(())
    }
}

