crate::ix!();

/// A fully functional mock implementing the same `CrateHandleInterface<P>`
/// as the real CrateHandle, but with configurable behaviors.
/// 
/// The goal is to simulate a crate's contents (like `src/main.rs`, `README.md`, etc.)
/// as well as the embedded `CargoTomlInterface` without touching the real filesystem.
#[derive(Builder, MutGetters, Getters, Debug, Clone)]
#[builder(setter(into))]
#[getset(get = "pub", get_mut = "pub")]
pub struct MockCrateHandle {
    /// A mock notion of where this crate is located. Often used as a "root" path.
    crate_path: PathBuf,

    /// The "name" returned by `Named::name()`.
    crate_name: String,

    /// The "version" returned by `Versioned::version()`.
    crate_version: String,

    /// If `is_private` is `true`, `is_private()` returns `Ok(true)`. Otherwise, `Ok(false)`.
    is_private_crate: bool,

    /// If `simulate_invalid_version` is `true`, then calls to `version()` will fail
    /// with a `CrateError::InvalidVersionFormat` or similar.
    simulate_invalid_version: bool,

    /// If `simulate_missing_main_or_lib` is `true`, calls to
    /// `check_src_directory_contains_valid_files()` will fail,
    /// simulating that there's no main.rs nor lib.rs.
    simulate_missing_main_or_lib: bool,

    /// If `simulate_missing_readme` is `true`, calls to `check_readme_exists()` will fail,
    /// simulating that there's no README.md.
    simulate_missing_readme: bool,

    /// If `simulate_no_tests_directory` is `true`, `has_tests_directory()` returns false,
    /// and any attempts to list test files may return an empty list or error.
    simulate_no_tests_directory: bool,

    /// If `simulate_failed_integrity` is `true`, calls to `validate_integrity()` fail directly,
    /// simulating a failing integrity check for any reason you choose.
    simulate_failed_integrity: bool,

    /// A mock table of "file path -> file contents" for `read_file_string`.
    /// If a path is not found in this map, we return an error in `read_file_string`.
    #[builder(default = "HashMap::new()")]
    file_contents: HashMap<PathBuf, String>,

    /// A mock list of source files for `source_files_excluding` usage.
    /// We do a simple path-based or filename-based approach.
    /// (In a real scenario, you might store them in `file_contents` as well.)
    #[builder(default = "Vec::new()")]
    source_files: Vec<PathBuf>,

    /// A mock list of test files for `test_files`.
    #[builder(default = "Vec::new()")]
    test_files: Vec<PathBuf>,

    /// An embedded mock of `CargoTomlInterface`. This can be a `MockCargoToml`
    /// or something that *itself* is fully configurable.
    /// We store it in an `Arc<Mutex<...>>` so we can hand it out in `HasCargoToml`.
    #[builder(default = "Arc::new(AsyncMutex::new(MockCargoToml::fully_valid_config()))")]
    mock_cargo_toml: Arc<AsyncMutex<MockCargoToml>>,
}

impl MockCrateHandle {
    /// A convenience constructor returning a "fully valid" mock crate:
    /// - Has a name "mock_crate"
    /// - Has version "1.2.3"
    /// - Not private
    /// - Integrity checks pass
    /// - Contains main.rs or lib.rs
    /// - Has a README
    /// - Has a tests directory
    /// - File reading will succeed for any path in `file_contents`
    /// - Has a fully valid `MockCargoToml` inside
    pub fn fully_valid_config() -> Self {
        trace!("MockCrateHandle::fully_valid_config constructor called");
        let mut file_map = HashMap::new();
        file_map.insert(PathBuf::from("README.md"), "# Mock Crate\n".into());
        file_map.insert(PathBuf::from("src/main.rs"), "// mock main".into());
        file_map.insert(PathBuf::from("tests/test_basic.rs"), "// mock test".into());

        MockCrateHandleBuilder::default()
            .crate_path("fake/mock/crate/path")
            .crate_name("mock_crate")
            .crate_version("1.2.3")
            .is_private_crate(false)
            .simulate_invalid_version(false)
            .simulate_missing_main_or_lib(false)
            .simulate_missing_readme(false)
            .simulate_no_tests_directory(false)
            .simulate_failed_integrity(false)
            .file_contents(file_map)
            .source_files(vec![PathBuf::from("src/main.rs")])
            .test_files(vec![PathBuf::from("tests/test_basic.rs")])
            // By default, embed a fully-valid MockCargoToml
            .mock_cargo_toml(Arc::new(AsyncMutex::new(MockCargoToml::fully_valid_config())))
            .build()
            .unwrap()
    }

    /// A constructor that simulates an invalid version scenario (Versioned trait fails).
    pub fn invalid_version_config() -> Self {
        trace!("MockCrateHandle::invalid_version_config constructor called");
        Self::fully_valid_config()
            .to_builder()
            .simulate_invalid_version(true)
            .build()
            .unwrap()
    }

    /// A constructor that simulates a crate missing main.rs/lib.rs.
    pub fn missing_main_or_lib_config() -> Self {
        trace!("MockCrateHandle::missing_main_or_lib_config constructor called");
        Self::fully_valid_config()
            .to_builder()
            .simulate_missing_main_or_lib(true)
            .build()
            .unwrap()
    }

    /// A constructor that simulates a crate missing its README.md.
    pub fn missing_readme_config() -> Self {
        trace!("MockCrateHandle::missing_readme_config constructor called");
        // We'll remove "README.md" from file_contents too
        let mut mc = Self::fully_valid_config();
        mc.file_contents_mut().remove(&PathBuf::from("README.md"));
        mc = mc
            .to_builder()
            .simulate_missing_readme(true)
            .build()
            .unwrap();
        mc
    }

    /// A constructor that simulates no tests directory.
    pub fn no_tests_directory_config() -> Self {
        trace!("MockCrateHandle::no_tests_directory_config constructor called");
        // We'll remove any test files
        let mut mc = Self::fully_valid_config();
        mc.test_files_mut().clear();
        mc.file_contents_mut().remove(&PathBuf::from("tests/test_basic.rs"));
        mc = mc
            .to_builder()
            .simulate_no_tests_directory(true)
            .build()
            .unwrap();
        mc
    }

    /// A constructor that simulates the crate being private.
    pub fn private_crate_config() -> Self {
        trace!("MockCrateHandle::private_crate_config constructor called");
        Self::fully_valid_config()
            .to_builder()
            .is_private_crate(true)
            .build()
            .unwrap()
    }

    /// A constructor that simulates an overall integrity failure (for any reason).
    pub fn failed_integrity_config() -> Self {
        trace!("MockCrateHandle::failed_integrity_config constructor called");
        Self::fully_valid_config()
            .to_builder()
            .simulate_failed_integrity(true)
            .build()
            .unwrap()
    }

    /// Helper for using the builder pattern on an existing instance (so we can mutate some fields).
    pub fn to_builder(&self) -> MockCrateHandleBuilder {
        let mut builder = MockCrateHandleBuilder::default();

        // Copy all fields from self into the builder
        builder
            .crate_path(self.crate_path().clone())
            .crate_name(self.crate_name().clone())
            .crate_version(self.crate_version().clone())
            .is_private_crate(*self.is_private_crate())
            .simulate_invalid_version(*self.simulate_invalid_version())
            .simulate_missing_main_or_lib(*self.simulate_missing_main_or_lib())
            .simulate_missing_readme(*self.simulate_missing_readme())
            .simulate_no_tests_directory(*self.simulate_no_tests_directory())
            .simulate_failed_integrity(*self.simulate_failed_integrity())
            .file_contents(self.file_contents().clone())
            .source_files(self.source_files().clone())
            .test_files(self.test_files().clone())
            .mock_cargo_toml(self.mock_cargo_toml().clone());

        builder
    }
}

// ----------------------------------------------------------------------
// Implement all the traits for MockCrateHandle
// ----------------------------------------------------------------------

impl Named for MockCrateHandle {
    fn name(&self) -> Cow<'_, str> {
        Cow::Owned(self.crate_name().clone())
    }
}

impl Versioned for MockCrateHandle {
    type Error = CrateError;

    fn version(&self) -> Result<semver::Version, Self::Error> {
        trace!("MockCrateHandle::version called");
        if *self.simulate_invalid_version() {
            error!("MockCrateHandle: simulating invalid version parse error");
            return Err(CrateError::SimulatedInvalidVersionFormat);
        }
        let parsed = semver::Version::parse(&self.crate_version()).map_err(|e| Arc::new(e))?;
        info!("MockCrateHandle: returning semver={}", parsed);
        Ok(parsed)
    }
}

#[async_trait]
impl IsPrivate for MockCrateHandle {
    type Error = CrateError;

    async fn is_private(&self) -> Result<bool, Self::Error> {
        trace!("MockCrateHandle::is_private called");
        Ok(*self.is_private_crate())
    }
}

#[async_trait]
impl ReadFileString for MockCrateHandle {
    async fn read_file_string(&self, path: &Path) -> Result<String, CrateError> {
        trace!("MockCrateHandle::read_file_string called with path={:?}", path);

        // Convert to an owned PathBuf to do lookups
        // If it's absolute or something, we still just do a map lookup.
        let path_buf = path.to_path_buf();

        // If the exact key is found, great. Otherwise, let's see if the path is relative to crate_path.
        if let Some(contents) = self.file_contents().get(&path_buf) {
            debug!("MockCrateHandle: found file contents by exact match in file_contents map");
            return Ok(contents.clone());
        }

        // Otherwise, try joining with crate_path if it's not absolute
        if !path_buf.is_absolute() {
            let joined = self.crate_path().join(&path_buf);
            if let Some(contents) = self.file_contents().get(&joined) {
                debug!("MockCrateHandle: found file contents by joined path");
                return Ok(contents.clone());
            }
        }

        // If not found in the map, simulate "file not found" or a read error
        error!(
            "MockCrateHandle: no entry in file_contents for path={:?}, read failed",
            path_buf
        );
        Err(CrateError::IoError {
            io_error: Arc::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found in MockCrateHandle",
            )),
            context: format!("Cannot read file at {:?}", path_buf),
        })
    }
}

impl CheckIfSrcDirectoryContainsValidFiles for MockCrateHandle {
    fn check_src_directory_contains_valid_files(&self) -> Result<(), CrateError> {
        trace!("MockCrateHandle::check_src_directory_contains_valid_files called");
        if *self.simulate_missing_main_or_lib() {
            error!("MockCrateHandle: simulating missing main.rs/lib.rs scenario");
            return Err(CrateError::FileNotFound {
                missing_file: self.crate_path().join("src").join("main.rs or lib.rs"),
            });
        }
        info!("MockCrateHandle: main.rs or lib.rs is considered present");
        Ok(())
    }
}

impl CheckIfReadmeExists for MockCrateHandle {
    fn check_readme_exists(&self) -> Result<(), CrateError> {
        trace!("MockCrateHandle::check_readme_exists called");
        if *self.simulate_missing_readme() {
            error!("MockCrateHandle: simulating missing README.md");
            return Err(CrateError::FileNotFound {
                missing_file: self.crate_path().join("README.md"),
            });
        }
        info!("MockCrateHandle: README.md is considered present");
        Ok(())
    }
}

#[async_trait]
impl GetReadmePath for MockCrateHandle {
    async fn readme_path(&self) -> Result<Option<PathBuf>, CrateError> {
        trace!("MockCrateHandle::readme_path called");
        if *self.simulate_missing_readme() {
            warn!("MockCrateHandle: README not present");
            Ok(None)
        } else {
            let path = self.crate_path().join("README.md");
            info!("MockCrateHandle: returning Some({:?})", path);
            Ok(Some(path))
        }
    }
}

#[async_trait]
impl GetSourceFilesWithExclusions for MockCrateHandle {
    async fn source_files_excluding(&self, exclude_files: &[&str]) -> Result<Vec<PathBuf>, CrateError> {
        trace!(
            "MockCrateHandle::source_files_excluding called, exclude_files={:?}",
            exclude_files
        );
        let mut results = vec![];
        for f in self.source_files().iter() {
            let file_name = f.file_name().and_then(|ff| ff.to_str()).unwrap_or("");
            if !exclude_files.contains(&file_name) {
                results.push(f.clone());
            }
        }
        Ok(results)
    }
}

#[async_trait]
impl GetTestFiles for MockCrateHandle {
    async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError> {
        trace!("MockCrateHandle::test_files called");
        if *self.simulate_no_tests_directory() {
            // We could either return an empty list or an error. For now, let's just return empty.
            info!("MockCrateHandle: simulating no tests directory => returning empty");
            Ok(vec![])
        } else {
            Ok(self.test_files().clone())
        }
    }
}

impl HasTestsDirectory for MockCrateHandle {
    fn has_tests_directory(&self) -> bool {
        trace!("MockCrateHandle::has_tests_directory called");
        !self.simulate_no_tests_directory()
    }
}

#[async_trait]
impl GetFilesInDirectory for MockCrateHandle {
    async fn get_files_in_dir(
        &self,
        dir_name: &str,
        _extension: &str,
    ) -> Result<Vec<PathBuf>, CrateError> {
        trace!("MockCrateHandle::get_files_in_dir called, dir_name={}", dir_name);
        // We'll just unify with get_files_in_dir_with_exclusions and pass an empty exclude list.
        let results = self
            .get_files_in_dir_with_exclusions(dir_name, _extension, &[])
            .await?;
        Ok(results)
    }
}

#[async_trait]
impl GetFilesInDirectoryWithExclusions for MockCrateHandle {
    async fn get_files_in_dir_with_exclusions(
        &self,
        dir_name: &str,
        extension: &str,
        exclude_files: &[&str],
    ) -> Result<Vec<PathBuf>, CrateError> {
        trace!(
            "MockCrateHandle::get_files_in_dir_with_exclusions called, dir_name={}, extension={}, exclude_files={:?}",
            dir_name,
            extension,
            exclude_files
        );

        // We'll gather from either source_files or test_files if they start with that dir
        // or we can do a naive approach if you want. We'll do a simple approach here:
        let mut results = vec![];
        for f in self.source_files().iter().chain(self.test_files().iter()) {
            let rel_str = f.to_string_lossy();
            if rel_str.contains(dir_name) {
                // Check extension
                if f.extension().and_then(|ex| ex.to_str()) == Some(extension) {
                    let file_name = f.file_name().and_then(|ff| ff.to_str()).unwrap_or("");
                    if !exclude_files.contains(&file_name) {
                        results.push(f.clone());
                    }
                }
            }
        }
        info!(
            "MockCrateHandle: returning {} file(s) for dir_name={}",
            results.len(),
            dir_name
        );
        Ok(results)
    }
}

impl HasCargoToml for MockCrateHandle {
    fn cargo_toml(&self) -> Arc<AsyncMutex<dyn CargoTomlInterface>> {
        trace!("MockCrateHandle::cargo_toml called");
        self.mock_cargo_toml().clone()
    }
}

impl AsRef<Path> for MockCrateHandle {
    fn as_ref(&self) -> &Path {
        trace!("MockCrateHandle::as_ref called, returning crate_path={:?}", self.crate_path());
        self.crate_path()
    }
}

#[async_trait]
impl GatherBinTargetNames for MockCrateHandle {
    type Error = CrateError;

    async fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error> {
        trace!("MockCrateHandle::gather_bin_target_names called");
        // For the mock, let's just delegate to the embedded MockCargoToml's gather_bin_target_names
        let bin_list = self.mock_cargo_toml()
            .lock()
            .await
            .gather_bin_target_names()
            .await?;

        Ok(bin_list)
    }
}

#[async_trait]
impl ValidateIntegrity for MockCrateHandle {
    type Error = CrateError;

    async fn validate_integrity(&self) -> Result<(), Self::Error> {
        trace!("MockCrateHandle::validate_integrity called");
        if *self.simulate_failed_integrity() {
            error!("MockCrateHandle: simulating overall integrity failure");
            return Err(CrateError::SimulatedIntegrityFailureInMockCrate);
        }
        // Otherwise, do some checks
        self.check_src_directory_contains_valid_files()?;
        self.check_readme_exists()?;
        // We might also check the embedded cargo toml's `validate_integrity` if we want
        // For now, let's skip or do it:
        self.mock_cargo_toml()
            .lock()
            .await
            .validate_integrity()
            .await
            .map_err(|e| CrateError::CargoTomlError(e))?;

        info!("MockCrateHandle: integrity validation passed");
        Ok(())
    }
}

// ----------------------------------------------------------------------
// Implement `AsyncTryFrom<P>` for MockCrateHandle
// This is needed to satisfy `CrateHandleInterface<P>`.
// In real usage, you might load from some config, but here we simulate all.
//
// We'll just *ignore* the input parameter and return `Ok(Self::fully_valid_config())`
// or we can do something else. We'll do a simple approach for demonstration:
// ----------------------------------------------------------------------
#[async_trait]
impl<P> AsyncTryFrom<P> for MockCrateHandle
where
    for<'async_trait> P: HasCargoTomlPathBuf + HasCargoTomlPathBufSync + AsRef<Path> + Send + Sync + 'async_trait,
    CrateError: From<<P as HasCargoTomlPathBuf>::Error>
        + From<<P as HasCargoTomlPathBufSync>::Error>,
{
    type Error = CrateError;

    async fn new(_crate_path: &P) -> Result<Self, Self::Error> {
        trace!("MockCrateHandle::AsyncTryFrom::new called for a mock handle");
        // For demonstration, we simply return a fully valid config.
        // If you want to adjust based on `_crate_path`, you can do so.
        Ok(MockCrateHandle::fully_valid_config())
    }
}

// ----------------------------------------------------------------------
// Implement the aggregator trait `CrateHandleInterface<P>`
// ----------------------------------------------------------------------
impl<P> CrateHandleInterface<P> for MockCrateHandle
where
    for<'async_trait> P: HasCargoTomlPathBuf + HasCargoTomlPathBufSync + AsRef<Path> + Send + Sync + 'async_trait,
    CrateError: From<<P as HasCargoTomlPathBuf>::Error>
        + From<<P as HasCargoTomlPathBufSync>::Error>,
{}

// ----------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------
#[cfg(test)]
mod tests_mock_crate_handle {
    use super::*;

    // Basic test demonstrating usage of the default "fully_valid_config"
    // and verifying that the mock calls work as expected.
    #[traced_test]
    async fn test_fully_valid_config_behaves_correctly() {
        let mock = MockCrateHandle::fully_valid_config();

        // 1) name() and version()
        assert_eq!(mock.name(), "mock_crate");
        let ver = mock.version().expect("Should parse version 1.2.3");
        assert_eq!(ver.to_string(), "1.2.3");

        // 2) is_private => false
        let priv_check = mock.is_private().await.unwrap();
        assert!(!priv_check, "Expected is_private_crate = false");

        // 3) read_file_string => can we read "README.md" from the map?
        let readme_contents = mock.read_file_string(Path::new("README.md")).await
            .expect("Should find README.md in file_contents");
        assert!(readme_contents.contains("# Mock Crate"));

        // 4) check_src_directory_contains_valid_files => no error
        mock.check_src_directory_contains_valid_files().expect("Should pass, as we have main.rs or lib.rs");

        // 5) check_readme_exists => no error
        mock.check_readme_exists().expect("Should pass, as we have README.md");

        // 6) gather_bin_target_names => delegates to embedded MockCargoToml
        // By default, the embedded MockCargoToml is "fully_valid_config" which might have no bin targets,
        // so let's see if it's empty:
        let bin_targets = mock.gather_bin_target_names().await.unwrap();
        assert!(bin_targets.len() == 1, "Default fully_valid_config from MockCargoToml has a single bin target");
    }

    #[traced_test]
    fn test_invalid_version_config_fails_versioned_trait() {
        let mock = MockCrateHandle::invalid_version_config();
        let ver_res = mock.version();
        assert!(ver_res.is_err(), "Should fail version parse");
    }

    #[traced_test]
    fn test_missing_main_or_lib_config_fails_src_check() {
        let mock = MockCrateHandle::missing_main_or_lib_config();
        let src_check = mock.check_src_directory_contains_valid_files();
        assert!(src_check.is_err(), "Should fail because we simulate missing main.rs/lib.rs");
    }

    #[traced_test]
    fn test_missing_readme_config_fails_readme_check() {
        let mock = MockCrateHandle::missing_readme_config();
        let readme_check = mock.check_readme_exists();
        assert!(readme_check.is_err(), "Should fail because we simulate missing README.md");
    }

    #[traced_test]
    fn test_no_tests_directory_config() {
        let mock = MockCrateHandle::no_tests_directory_config();
        assert!(!mock.has_tests_directory(), "Should simulate no tests directory");
        let test_files = mock.test_files();
        assert!(test_files.is_empty(), "No test files in this config");
    }

    #[traced_test]
    async fn test_private_crate_config_returns_true_for_is_private() {
        let mock = MockCrateHandle::private_crate_config();
        let priv_check = mock.is_private().await.unwrap();
        assert!(priv_check, "Should be private");
    }

    #[traced_test]
    async fn test_failed_integrity_config() {
        let mock = MockCrateHandle::failed_integrity_config();
        let integrity_res = mock.validate_integrity().await;
        assert!(integrity_res.is_err(), "Simulated integrity failure");
    }

    #[traced_test]
    async fn test_read_file_string_map_lookup() {
        let mut file_map = HashMap::new();
        file_map.insert(PathBuf::from("src/main.rs"), "fn main() {}".into());
        let mock = MockCrateHandle::fully_valid_config()
            .to_builder()
            .file_contents(file_map)
            .build()
            .unwrap();

        let contents = mock.read_file_string(Path::new("src/main.rs")).await
            .expect("Should find main.rs in the file map");
        assert_eq!(contents, "fn main() {}");

        // If we try to read a file not in the map, we get an error
        let missing_res = mock.read_file_string(Path::new("non_existent_file.txt")).await;
        assert!(missing_res.is_err(), "Expect an IoError for missing file path in the map");
    }
}
