// ---------------- [ File: workspacer-crate/src/crate_handle.rs ]
crate::ix!();

#[derive(Getters,Debug,Clone)]
#[getset(get="pub")]
pub struct CrateHandle {
    crate_path:        PathBuf,
    cargo_toml_handle: CargoToml,
}

impl<P> CrateHandleInterface<P> for CrateHandle 
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
impl<P> AsyncTryFrom<P> for CrateHandle 
where 
    for<'async_trait> 
    P
    : HasCargoTomlPathBuf 
    + AsRef<Path> 
    + Send 
    + Sync
    + 'async_trait,

    CrateError: From<<P as HasCargoTomlPathBuf>::Error>,
{
    type Error = CrateError;

    /// Initializes a crate handle from a given crate_path
    async fn new(crate_path: &P) -> Result<Self,Self::Error> {

        let cargo_toml_path = crate_path.cargo_toml_path_buf().await?;

        let cargo_toml_handle = CargoToml::new(cargo_toml_path).await?;

        Ok(Self {
            cargo_toml_handle,
            crate_path: crate_path.as_ref().to_path_buf(),
        })
    }
}

impl ValidateIntegrity for CrateHandle {

    type Error = CrateError;

    /// Validates the integrity of a crate by checking required files and directory structure
    fn validate_integrity(&self) -> Result<(), Self::Error> {

        let cargo_toml = self.cargo_toml();

        cargo_toml.validate_integrity()?;

        self.check_src_directory_contains_valid_files()?;
        self.check_readme_exists()?;

        Ok(())
    }
}

impl CheckIfSrcDirectoryContainsValidFiles for CrateHandle {

    /// Checks if the `src/` directory contains a `lib.rs` or `main.rs`
    fn check_src_directory_contains_valid_files(&self) -> Result<(), CrateError> {
        let src_dir = self.as_ref().join("src");
        let main_rs = src_dir.join("main.rs");
        let lib_rs = src_dir.join("lib.rs");

        if !main_rs.exists() && !lib_rs.exists() {
            return Err(CrateError::FileNotFound {
                missing_file: src_dir.join("main.rs or lib.rs"),
            });
        }

        // It's okay if both exist
        Ok(())
    }
}

impl CheckIfReadmeExists for CrateHandle {

    /// Checks if `README.md` exists
    fn check_readme_exists(&self) -> Result<(), CrateError> {
        let readme_path = self.as_ref().join("README.md");
        if !readme_path.exists() {
            return Err(CrateError::FileNotFound {
                missing_file: readme_path,
            });
        }
        Ok(())
    }
}

#[async_trait]
impl GetReadmePath for CrateHandle {

    /// Asynchronously returns the path to the `README.md` if it exists
    async fn readme_path(&self) -> Result<Option<PathBuf>, CrateError> {
        let readme_path = self.crate_path.join("README.md");
        if fs::metadata(&readme_path).await.is_ok() {
            Ok(Some(readme_path))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl GetSourceFilesWithExclusions for CrateHandle {

    /// Asynchronously returns a list of source files (`.rs`) in the `src/` directory, excluding specified files
    async fn source_files_excluding(&self, exclude_files: &[&str]) -> Result<Vec<PathBuf>, CrateError> {
        self.get_files_in_dir_with_exclusions("src", "rs", exclude_files).await
    }
}

#[async_trait]
impl GetTestFiles for CrateHandle {

    /// Asynchronously returns a list of test files (`.rs`) in the `tests/` directory
    async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError> {
        self.get_files_in_dir("tests", "rs").await
    }
}

impl HasTestsDirectory for CrateHandle {

    fn has_tests_directory(&self) -> bool {
        self.crate_path.join("tests").exists()
    }
}

#[async_trait]
impl GetFilesInDirectory for CrateHandle {

    /// Asynchronously returns a list of files with the given extension in the specified directory
    async fn get_files_in_dir(&self, dir_name: &str, extension: &str) -> Result<Vec<PathBuf>, CrateError> {
        self.get_files_in_dir_with_exclusions(dir_name, extension, &[]).await
    }
}

#[async_trait]
impl GetFilesInDirectoryWithExclusions for CrateHandle {

    /// Asynchronously returns a list of files with the given extension in the specified directory,
    /// excluding specified file names.
    async fn get_files_in_dir_with_exclusions(
        &self,
        dir_name: &str,
        extension: &str,
        exclude_files: &[&str]
    ) -> Result<Vec<PathBuf>, CrateError> {
        let dir_path = self.crate_path.join(dir_name);

        if !fs::metadata(&dir_path).await.is_ok() {
            return Err(CrateError::DirectoryNotFound {
                missing_directory: dir_path,
            });
        }

        let mut files = vec![];

        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(|e| DirectoryError::ReadDirError {io: e.into() })?;

        while let Some(entry) 
            = entries.next_entry()
            .await
            .map_err(|e| DirectoryError::GetNextEntryError {io: e.into() })? 
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                CrateError::FailedToGetFileNameForPath {
                    path: path.to_path_buf()
                }
            })?;

            if path.extension().and_then(|e| e.to_str()) == Some(extension) && !exclude_files.contains(&file_name) {
                files.push(path);
            }
        }

        Ok(files)
    }
}

impl HasCargoToml for CrateHandle {

    fn cargo_toml<'a>(&'a self) -> &'a dyn CargoTomlInterface {
        &self.cargo_toml_handle
    }
}

impl AsRef<Path> for CrateHandle {
    /// Allows CrateHandle to be used as a path by referencing crate_path
    fn as_ref(&self) -> &Path {
        &self.crate_path
    }
}

#[async_trait]
impl ReadyForCargoPublish for CrateHandle {

    type Error = CrateError;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {

        let cargo_toml = self.cargo_toml();
        cargo_toml.ready_for_cargo_publish().await?;

        self.check_readme_exists()?;
        self.check_src_directory_contains_valid_files()?;

        Ok(())
    }
}
