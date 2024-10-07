crate::ix!();

#[derive(Debug,Deserialize,Serialize,Builder)]
#[builder(pattern = "owned")]
pub struct Repo {
    name:   String,
    url:    String,
    path:   String,
    branch: String,
    remote: String,
}

impl Named for Repo {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

impl Repo {

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub fn remote(&self) -> &str {
        &self.remote
    }
}

impl GitStatusClean for Repo {

    fn git_status_clean(&self) -> Result<bool, git2::Error> {
        let repo_path = Path::new(&self.path);
        if repo_path.exists() && repo_path.is_dir() {
            let repository = Repository::open(repo_path)?;
            let statuses = repository.statuses(None)?;
            let is_clean = statuses.is_empty();
            Ok(is_clean)
        } else {
            warn!("{} does not exist", self.path);
            Ok(false)
        }
    }
}

impl IsPushedUpstream for Repo {

    fn is_pushed_upstream(&self) -> Result<bool, git2::Error> {
        let repo_path = Path::new(&self.path);
        if repo_path.exists() && repo_path.is_dir() {
            let repository = Repository::open(repo_path)?;
            let local_branch = repository.find_branch(&self.branch, git2::BranchType::Local)?;
            let local_commit = local_branch.get().peel_to_commit()?;

            // Get the remote tracking branch
            let remote_branch = format!("{}/{}", self.remote, self.branch);
            let remote_commit = repository.revparse_single(&format!("refs/remotes/{}", remote_branch))?.peel_to_commit()?;

            // Compare commits
            Ok(local_commit.id() == remote_commit.id())
        } else {
            warn!("{} does not exist", self.path);
            Ok(false)
        }
    }
}

impl GitSync for Repo {

    fn git_sync(&self) -> Result<(), git2::Error> {
        let repo_path = Path::new(&self.path);
        let repository = Repository::open(repo_path)?;

        let mut remote = repository.find_remote(&self.remote)?;

        // Fetch from the remote
        remote.fetch(&[&self.branch], None, None)?;

        // Perform a fast-forward merge
        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let fetch_commit = repository.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repository.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_fast_forward() {
            let mut reference = repository.find_reference(&format!("refs/heads/{}", self.branch))?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repository.set_head(&format!("refs/heads/{}", self.branch))?;
            repository.checkout_head(None)?;
        }

        warn!("{} synced to the latest commit", self.name);
        Ok(())
    }
}
