// ---------------- [ File: workspacer-interface/src/crate_handle_interface.rs ]
crate::ix!();

pub trait CrateHandleInterface<P>
: ValidateIntegrity<Error=CrateError>
+ Send
+ Sync
+ EnsureGitClean<Error=GitError>
+ PinWildcardDependencies<Error=CrateError>
+ PinAllWildcardDependencies<Error=CrateError>
+ ReadyForCargoPublish<Error=CrateError>
+ CheckIfSrcDirectoryContainsValidFiles
+ CheckIfReadmeExists
+ GetReadmePath
+ GetSourceFilesWithExclusions
+ GetTestFiles
+ HasTestsDirectory
+ GetFilesInDirectory
+ GetFilesInDirectoryWithExclusions
+ HasCargoToml
+ AsRef<Path>
+ AsyncTryFrom<P,Error=CrateError>
where
    for<'async_trait> 
    P
    : HasCargoTomlPathBuf 
    + AsRef<Path> 
    + Send 
    + Sync
    + 'async_trait,

    CrateError: From<<P as HasCargoTomlPathBuf>::Error>,
{}

#[async_trait]
pub trait GetTestFiles {

    async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError>;
}

pub trait HasTestsDirectory {

    fn has_tests_directory(&self) -> bool;
}

pub trait CheckIfReadmeExists {

    fn check_readme_exists(&self) -> Result<(), CrateError>;
}

#[async_trait]
pub trait GetReadmePath {

    async fn readme_path(&self) -> Result<Option<PathBuf>, CrateError>;
}

pub trait HasCargoToml {

    fn cargo_toml<'a>(&'a self) -> &'a dyn CargoTomlInterface;
}

#[async_trait]
pub trait HasCargoTomlPathBuf {

    type Error;

    async fn cargo_toml_path_buf(&self) -> Result<PathBuf, Self::Error>;
}

#[async_trait]
impl<P> HasCargoTomlPathBuf for P 
where for <'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = CrateError;

    /// Asynchronously returns the path to the `Cargo.toml`
    async fn cargo_toml_path_buf(&self) -> Result<PathBuf, Self::Error> 
    {
        let cargo_path = self.as_ref().join("Cargo.toml");
        if fs::metadata(&cargo_path).await.is_ok() {
            Ok(cargo_path)
        } else {
            Err(CrateError::FileNotFound {
                missing_file: cargo_path,
            })
        }
    }
}

pub trait CheckIfSrcDirectoryContainsValidFiles {

    fn check_src_directory_contains_valid_files(&self) -> Result<(), CrateError>;
}

#[async_trait]
pub trait GetSourceFilesWithExclusions {

    async fn source_files_excluding(&self, exclude_files: &[&str]) -> Result<Vec<PathBuf>, CrateError>;
}

#[async_trait]
pub trait GetFilesInDirectory {

    async fn get_files_in_dir(&self, dir_name: &str, extension: &str) 
        -> Result<Vec<PathBuf>, CrateError>;
}

#[async_trait]
pub trait GetFilesInDirectoryWithExclusions {

    async fn get_files_in_dir_with_exclusions(
        &self,
        dir_name: &str,
        extension: &str,
        exclude_files: &[&str]
    ) -> Result<Vec<PathBuf>, CrateError>;
}

/// Trait for checking if a component is ready for Cargo publishing
#[async_trait]
pub trait ReadyForCargoPublish {

    type Error;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error>;
}
