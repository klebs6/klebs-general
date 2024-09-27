crate::ix!();

#[async_trait]
pub trait GetTestFiles {

    async fn test_files(&self) -> Result<Vec<PathBuf>, WorkspaceError>;
}

pub trait HasTestsDirectory {

    fn has_tests_directory(&self) -> bool;
}
