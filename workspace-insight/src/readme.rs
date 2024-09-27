crate::ix!();

pub trait CheckIfReadmeExists {

    fn check_readme_exists(&self) -> Result<(), WorkspaceError>;
}

#[async_trait]
pub trait GetReadmePath {

    async fn readme_path(&self) -> Result<Option<PathBuf>, WorkspaceError>;
}
