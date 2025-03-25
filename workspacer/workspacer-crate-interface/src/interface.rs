// ---------------- [ File: workspacer-crate-interface/src/interface.rs ]
crate::ix!();

pub trait CrateHandleInterface<P>
: ValidateIntegrity<Error=CrateError>
+ Send
+ Sync
+ Debug
+ Named
+ Versioned<Error=CrateError>
+ IsPrivate<Error=CrateError>
+ ReadFileString
+ CheckIfSrcDirectoryContainsValidFiles
+ CheckIfReadmeExists
+ GetReadmePath
+ GetSourceFilesWithExclusions
+ GetTestFiles
+ HasTestsDirectory
+ GetFilesInDirectory
+ GetFilesInDirectoryWithExclusions
+ HasCargoToml
+ RootDirPathBuf
+ AsRef<Path>
+ GatherBinTargetNames<Error=CrateError>
+ AsyncTryFrom<P,Error=CrateError>
where
    for<'async_trait> 
    P
    : HasCargoTomlPathBuf 
    + HasCargoTomlPathBufSync
    + AsRef<Path> 
    + Send 
    + Sync
    + 'async_trait,

    CrateError
    : From<<P as HasCargoTomlPathBuf>::Error> 
    + From<<P as HasCargoTomlPathBufSync>::Error>,
{}

pub trait HasCargoToml {

    fn cargo_toml(&self) -> Arc<AsyncMutex<dyn CargoTomlInterface>>;
}

#[async_trait]
pub trait IsPrivate {
    type Error;
    async fn is_private(&self) -> Result<bool,Self::Error>;
}

/// We add a new method to CrateHandleInterface so we can read file text from 
/// an in-memory mock or from the real filesystem. For your real code, 
/// you might implement it differently.
#[async_trait]
pub trait ReadFileString {
    async fn read_file_string(&self, path: &Path) -> Result<String, CrateError>;
}

#[async_trait]
pub trait GetTestFiles {

    async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError>;
}

#[async_trait]
impl<P> GetTestFiles for Arc<AsyncMutex<P>>
where
    P: GetTestFiles + Send + Sync,
{
    async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError> {
        let guard = self.lock().await;
        guard.test_files().await
    }
}

pub trait HasTestsDirectory {

    fn has_tests_directory(&self) -> bool;
}

impl<P> HasTestsDirectory for Arc<AsyncMutex<P>>
where
    P: HasTestsDirectory,
{
    fn has_tests_directory(&self) -> bool {
        // We only need a synchronous lock here because HasTestsDirectory's method is sync,
        // but our crate uses `tokio::sync::AsyncMutex`, so we do `.blocking_lock()`
        // or we can do `.try_lock()`, depending on usage. If you want the async version,
        // you might have to change the trait or store that state differently.
        // For simplicity, we'll do `.try_lock()` and fallback to false if locked.
        // Ideally, you'd define HasTestsDirectory as async, or
        // use the feature "blocking" from `tokio`.
        if let Ok(guard) = self.try_lock() {
            guard.has_tests_directory()
        } else {
            // If for some reason it's locked, we might do something else. 
            // For test usage, it's probably safe to do a separate approach:
            panic!("Cannot lock Arc<AsyncMutex<P>> in has_tests_directory synchronously! Consider an async method or a blocking lock feature.")
        }
    }
}

pub trait CheckIfReadmeExists {

    fn check_readme_exists(&self) -> Result<(), CrateError>;
}

#[async_trait]
pub trait GetReadmePath {

    async fn readme_path(&self) -> Result<Option<PathBuf>, CrateError>;
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

pub trait HasCargoTomlPathBufSync {

    type Error;

    fn cargo_toml_path_buf_sync(&self) -> Result<PathBuf, Self::Error>;
}

impl<P> HasCargoTomlPathBufSync for P 
where P: AsRef<Path>
{
    type Error = CrateError;

    /// Asynchronously returns the path to the `Cargo.toml`
    fn cargo_toml_path_buf_sync(&self) -> Result<PathBuf, Self::Error> 
    {
        let cargo_path = self.as_ref().join("Cargo.toml");
        if std::fs::metadata(&cargo_path).is_ok() {
            Ok(cargo_path)
        } else {
            Err(CrateError::FileNotFound {
                missing_file: cargo_path,
            })
        }
    }
}


pub trait RootDirPathBuf {

    fn root_dir_path_buf(&self) -> PathBuf;
}

impl<P> RootDirPathBuf for P 
where for <'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait
{
    /// returns the path to the `Cargo.toml`
    fn root_dir_path_buf(&self) -> PathBuf
    {
        self.as_ref().to_path_buf()
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
impl<P> GetSourceFilesWithExclusions for Arc<AsyncMutex<P>>
where
    P: GetSourceFilesWithExclusions + Send + Sync,
{
    async fn source_files_excluding(
        &self,
        exclude: &[&str],
    ) -> Result<Vec<PathBuf>, CrateError> {
        // Lock and forward
        let guard = self.lock().await;
        guard.source_files_excluding(exclude).await
    }
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
