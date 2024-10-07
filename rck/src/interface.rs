crate::ix!();

pub trait GitStatusClean {

    fn git_status_clean(&self) -> Result<bool, git2::Error>;
}

pub trait IsPushedUpstream {

    fn is_pushed_upstream(&self) -> Result<bool, git2::Error>;
}

pub trait GitSync {

    fn git_sync(&self) -> Result<(), git2::Error>;
}

pub trait ProcessRepos {

    fn process_repos(&self, operation: RepoOperation) -> Result<(),git2::Error>;
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum RepoOperation {
    Sync,
    Status
}
