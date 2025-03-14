// ---------------- [ File: workspacer-crate/src/crate_handle.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Builder,Getters,Debug,Clone)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct CrateHandle {
    crate_path:        PathBuf,
    cargo_toml_handle: Arc<Mutex<CargoToml>>,
}

impl Named for CrateHandle {
    fn name(&self) -> Cow<'_, str> {
        Cow::Owned(
            self.cargo_toml_handle.lock().unwrap()
                .package_name()
                .expect("expect that our crate has a package name")
                .trim_matches('"')
                .to_string()
        )
    }
}

impl Versioned for CrateHandle {
    type Error = CrateError;

    fn version(&self) -> Result<semver::Version, Self::Error> {
        trace!("CrateHandle::version() - retrieving version via cargo_toml_handle");
        let mut version_str = self.cargo_toml_handle.lock().unwrap().version()?.to_string();
        debug!("Raw version_str from CargoTomlHandle: {:?}", version_str);

        // Clean/trim quotes if present
        version_str = version_str.trim().replace('"', "");
        debug!("Cleaned version_str: {:?}", version_str);

        // Parse as semver
        let parsed = semver::Version::parse(&version_str)
            .map_err(|e| {
                error!("Failed to parse semver from string='{}': {:?}", version_str, e);
                CrateError::CargoTomlError(CargoTomlError::SemverError(Arc::new(e)))
            })?;

        info!("CrateHandle::version() - final parsed version for {:?} => {}", self.as_ref(), parsed);
        Ok(parsed)
    }
}

impl<P> CrateHandleInterface<P> for CrateHandle 
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

#[async_trait]
impl<P> AsyncTryFrom<P> for CrateHandle 
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
{
    type Error = CrateError;

    /// Initializes a crate handle from a given crate_path
    async fn new(crate_path: &P) -> Result<Self,Self::Error> {

        let cargo_toml_path = crate_path.cargo_toml_path_buf().await?;

        let cargo_toml_handle = Arc::new(Mutex::new(CargoToml::new(cargo_toml_path).await?));

        Ok(Self {
            cargo_toml_handle,
            crate_path: crate_path.as_ref().to_path_buf(),
        })
    }
}

impl CrateHandle 
{
    /// Initializes a crate handle from a given crate_path
    pub fn new_sync<P>(crate_path: &P) -> Result<Self,CrateError> 
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
    {

        let cargo_toml_path = crate_path.cargo_toml_path_buf_sync()?;

        let cargo_toml_handle = Arc::new(Mutex::new(CargoToml::new_sync(cargo_toml_path)?));

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

        cargo_toml.lock().unwrap().validate_integrity()?;

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

    fn cargo_toml(&self) -> Arc<Mutex<dyn CargoTomlInterface>> {
        self.cargo_toml_handle.clone()
    }
}

impl AsRef<Path> for CrateHandle {
    /// Allows CrateHandle to be used as a path by referencing crate_path
    fn as_ref(&self) -> &Path {
        &self.crate_path
    }
}

#[cfg(test)]
mod test_crate_handle {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::{tempdir, TempDir};
    use tokio::fs::{File, create_dir_all};
    use tokio::io::AsyncWriteExt;

    // A small helper that creates and writes arbitrary text to a file.
    async fn write_file(file_path: &Path, content: &str) {
        if let Some(parent_dir) = file_path.parent() {
            create_dir_all(parent_dir)
                .await
                .expect("Failed to create parent directories");
        }
        let mut f = File::create(file_path)
            .await
            .unwrap_or_else(|e| panic!("Could not create file {}: {e}", file_path.display()));
        f.write_all(content.as_bytes())
            .await
            .unwrap_or_else(|e| panic!("Failed to write to file {}: {e}", file_path.display()));
    }

    // Creates a basic "Cargo.toml" content.  
    // By default, includes `[package] name, version, authors, license`.
    fn minimal_cargo_toml(name: &str, version: &str) -> String {
        format!(
            r#"[package]
name = "{name}"
version = "{version}"
authors = ["Some Body"]
license = "MIT"
"#,
        )
    }

    /// Helper to build a `CrateHandle` by placing a Cargo.toml file (and optional other files)
    /// in a temporary directory, then calling `CrateHandle::new(...)`.
    /// We return the TempDir too, so it stays alive while tests run.
    async fn create_crate_handle_in_temp(
        crate_name: &str,
        crate_version: &str,
        create_src_dir: bool,
        create_tests_dir: bool,
        create_readme: bool,
        main_or_lib: Option<&str>, // "main" or "lib" or None
    ) -> (TempDir, CrateHandle) {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let root_path = tmp_dir.path().to_path_buf();

        // Write Cargo.toml
        let cargo_toml_content = minimal_cargo_toml(crate_name, crate_version);
        let cargo_toml_path = root_path.join("Cargo.toml");
        write_file(&cargo_toml_path, &cargo_toml_content).await;

        // Optionally create src and main.rs or lib.rs
        if create_src_dir {
            if let Some(which) = main_or_lib {
                let file_name = format!("{which}.rs");
                let file_path = root_path.join("src").join(file_name);
                write_file(&file_path, "// sample content").await;
            }
        }

        // Optionally create tests directory
        if create_tests_dir {
            let test_file_path = root_path.join("tests").join("test_basic.rs");
            write_file(&test_file_path, "// test file content").await;
        }

        // Optionally create README.md
        if create_readme {
            let readme_path = root_path.join("README.md");
            write_file(&readme_path, "# My Crate\nSome description.").await;
        }

        // Minimal struct to implement `HasCargoTomlPathBuf`
        #[derive(Clone)]
        struct TempCratePath(PathBuf);

        impl AsRef<Path> for TempCratePath {
            fn as_ref(&self) -> &Path {
                self.0.as_ref()
            }
        }

        // Create the input object
        let temp_crate_path = TempCratePath(root_path.clone());

        // Finally call CrateHandle::new
        let handle = CrateHandle::new(&temp_crate_path)
            .await
            .expect("Failed to create CrateHandle from temp directory");

        (tmp_dir, handle)
    }

    // ------------------------------------------------------------------------
    // Actual tests
    // ------------------------------------------------------------------------

    /// 1) Test that name() and version() work for a minimal crate.
    #[tokio::test]
    async fn test_name_and_version() {
        let (_tmp_dir, handle) =
            create_crate_handle_in_temp("test_crate", "0.1.0", false, false, false, None).await;
        eprintln!("handle: {:#?}", handle);
        assert_eq!(handle.name(), "test_crate");
        eprintln!("handle.name(): {:#?}", handle.name());
        let ver = handle.version().expect("Expected valid version");
        eprintln!("handle.version(): {:#?}", handle.version());
        assert_eq!(ver.to_string(), "0.1.0");
    }

    /// 2) Test check_src_directory_contains_valid_files when we have src/main.rs
    #[tokio::test]
    async fn test_check_src_directory_contains_valid_files_main_rs() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,  // create src
            false, // no tests
            false, // no readme
            Some("main"),
        )
        .await;

        // Should not error
        handle.check_src_directory_contains_valid_files().expect("Should find main.rs");
    }

    /// 3) Test check_src_directory_contains_valid_files when we have neither main.rs nor lib.rs => error
    #[tokio::test]
    async fn test_check_src_directory_contains_valid_files_missing_main_and_lib() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,  // create src dir
            false, // no tests
            false, // no readme
            None,  // no main or lib
        )
        .await;

        let result = handle.check_src_directory_contains_valid_files();
        assert!(
            result.is_err(),
            "Expected an error because neither main.rs nor lib.rs is present"
        );
        match result {
            Err(CrateError::FileNotFound { missing_file }) => {
                let missing = missing_file.to_string_lossy();
                assert!(
                    missing.contains("main.rs or lib.rs"),
                    "Error message should mention main.rs or lib.rs"
                );
            }
            _ => panic!("Expected CrateError::FileNotFound with mention of main.rs or lib.rs"),
        }
    }

    /// 4) Test check_readme_exists => success when README.md is present
    #[tokio::test]
    async fn test_check_readme_exists_ok() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,   // src
            false,  // tests
            true,   // readme
            Some("lib"),
        )
        .await;

        // Should not error
        handle.check_readme_exists().expect("README.md should exist");
    }

    /// 5) Test check_readme_exists => error when no README.md
    #[tokio::test]
    async fn test_check_readme_exists_missing() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,  // src
            false, // tests
            false, // readme missing
            Some("lib"),
        )
        .await;

        let result = handle.check_readme_exists();
        assert!(result.is_err());
        match result {
            Err(CrateError::FileNotFound { missing_file }) => {
                let missing = missing_file.to_string_lossy();
                assert!(
                    missing.contains("README.md"),
                    "Expected error referencing README.md"
                );
            }
            _ => panic!("Expected CrateError::FileNotFound for missing README.md"),
        }
    }

    /// 6) Test has_tests_directory => false if we never created it, true if we did.
    #[tokio::test]
    async fn test_has_tests_directory() {
        let (_tmp_dir, handle_no_tests) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,
            false, // no tests
            false,
            Some("lib"),
        )
        .await;
        assert!(
            !handle_no_tests.has_tests_directory(),
            "Expected false, no tests/ folder"
        );

        let (_tmp_dir, handle_with_tests) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,
            true, // yes tests
            false,
            Some("lib"),
        )
        .await;
        assert!(
            handle_with_tests.has_tests_directory(),
            "Expected true, tests/ folder created"
        );
    }

    /// 7) Test get_source_files_excluding and get_test_files
    #[tokio::test]
    async fn test_file_enumeration_in_source_and_tests() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "mycrate",
            "0.1.0",
            true,  // create src
            true,  // create tests
            true,  // readme
            Some("lib"),
        )
        .await;

        // Add one more file in src
        let extra_src = handle.as_ref().join("src").join("extra.rs");
        write_file(&extra_src, "// extra file").await;

        // Add one more file in tests
        let extra_test = handle.as_ref().join("tests").join("extra_test.rs");
        write_file(&extra_test, "// extra test file").await;

        // Now ask for the source files
        let src_files = handle
            .source_files_excluding(&[])
            .await
            .expect("Should list src files");
        // We expect 2: lib.rs + extra.rs
        assert_eq!(src_files.len(), 2, "Should find 2 .rs files in src");

        // Now check test files
        let test_files = handle.test_files().await.expect("Should list test files");
        // We expect 2: test_basic.rs + extra_test.rs
        assert_eq!(test_files.len(), 2, "Should find 2 .rs files in tests");
    }

    /// 8) Test source_files_excluding to ensure we skip any excluded file(s).
    #[tokio::test]
    async fn test_source_files_excluding() {
        let (_tmp_dir, handle) = create_crate_handle_in_temp(
            "excluded_crate",
            "0.1.0",
            true,
            false,
            true,
            Some("lib"),
        )
        .await;

        // Add one more file in src
        let extra_src = handle.as_ref().join("src").join("exclude_me.rs");
        write_file(&extra_src, "// exclude me").await;

        // If we exclude "exclude_me.rs", we should only see "lib.rs"
        let src_files = handle
            .source_files_excluding(&["exclude_me.rs"])
            .await
            .unwrap();
        assert_eq!(src_files.len(), 1, "Expected to exclude exclude_me.rs");
        let only_file_name = src_files[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(only_file_name, "lib.rs");
    }

    /// Test validate_integrity => ensures the crate has Cargo.toml, a valid src file, and readme, etc.
    /// (check_src_directory_contains_valid_files + check_readme_exists).
    #[tokio::test]
    async fn test_validate_integrity() {
        // a) valid scenario
        let (_tmp_dir, handle_ok) = create_crate_handle_in_temp(
            "integrity_crate",
            "0.1.1",
            true,
            false,
            true,
            Some("lib"),
        )
        .await;
        let res_ok = handle_ok.validate_integrity();
        assert!(res_ok.is_ok(), "Expected valid integrity with a src file and README");

        // b) missing main.rs/lib.rs => should fail
        let (_tmp_dir, handle_bad_src) = create_crate_handle_in_temp(
            "bad_src_crate",
            "0.1.0",
            true,
            false,
            true,
            None, // no main/lib
        )
        .await;
        let res_bad_src = handle_bad_src.validate_integrity();
        assert!(
            res_bad_src.is_err(),
            "Expected integrity check to fail with missing main.rs/lib.rs"
        );

        // c) missing README => fail
        let (_tmp_dir, handle_no_readme) = create_crate_handle_in_temp(
            "no_readme_crate",
            "0.1.0",
            true,
            false,
            false,
            Some("main"),
        )
        .await;
        let res_no_readme = handle_no_readme.validate_integrity();
        assert!(res_no_readme.is_err(), "Expected missing README.md error");
        match res_no_readme {
            Err(CrateError::FileNotFound { missing_file }) => {
                assert!(
                    missing_file.ends_with("README.md"),
                    "Expected error referencing missing README.md"
                );
            }
            _ => panic!("Expected FileNotFound referencing README.md"),
        }
    }
}
