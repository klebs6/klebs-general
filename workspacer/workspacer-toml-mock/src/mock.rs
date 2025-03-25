crate::ix!();

/// A fully functional mock implementing the same `CargoTomlInterface`
/// as the real CargoToml, but with configurable behaviors.
#[derive(Builder, MutGetters, Getters, Debug, Clone)]
#[builder(setter(into))]
#[getset(get = "pub", get_mut = "pub")]
pub struct MockCargoToml {
    path: PathBuf,

    /// If `file_exists` is `false`, this mock simulates that the file does not exist.
    file_exists: bool,

    /// If `all_required_fields_for_publishing_present` is `false`, this mock simulates
    /// that required fields for publishing are missing.
    all_required_fields_for_publishing_present: bool,

    /// If `all_required_fields_for_integrity_present` is `false`, this mock simulates
    /// that required fields for integrity are missing.
    all_required_fields_for_integrity_present: bool,

    /// If `valid_version_for_publishing` is `false`, this mock simulates that
    /// the version is invalid for publishing.
    valid_version_for_publishing: bool,

    /// If `valid_version_for_integrity` is `false`, this mock simulates that
    /// the version is invalid for integrity.
    valid_version_for_integrity: bool,

    /// If `simulate_save_error` is `true`, this mock simulates an I/O error on save.
    simulate_save_error: bool,

    /// If `dependency_update_changes_something` is `true`, calls to
    /// `update_dependency_version` will return `Ok(true)` as if it changed a dependency.
    dependency_update_changes_something: bool,

    /// Mock authors for `get_package_authors`.
    #[builder(default)]
    package_authors: Option<Vec<String>>,

    /// Mock Rust edition for `get_rust_edition`.
    #[builder(default)]
    rust_edition: Option<String>,

    /// Mock license for `get_license_type`.
    #[builder(default)]
    license: Option<String>,

    /// Mock repository location for `get_crate_repository_location`.
    #[builder(default)]
    repository: Option<String>,

    /// Mock binary target names for `gather_bin_target_names`.
    #[builder(default)]
    bin_target_names: Vec<String>,
}

impl AsRef<Path> for MockCargoToml {
    fn as_ref(&self) -> &Path {
        trace!("MockCargoToml::as_ref called, returning path={:?}", self.path);
        &self.path
    }
}

#[async_trait]
impl SaveToDisk for MockCargoToml {
    type Error = CargoTomlError;

    async fn save_to_disk(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::save_to_disk called");
        if *self.simulate_save_error() {
            error!("MockCargoToml: simulating an I/O error on save_to_disk");
            let io_err = std::io::Error::new(std::io::ErrorKind::Other, "Simulated I/O error from mock");
            return Err(CargoTomlError::IoWriteError {
                path: self.path().clone(),
                source: Arc::new(io_err),
            });
        }
        info!("MockCargoToml: save_to_disk returning Ok");
        Ok(())
    }
}

impl UpdateDependencyVersionRaw for MockCargoToml {
    type Error = CargoTomlError;

    fn update_dependency_version(
        &mut self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<bool, Self::Error> {
        trace!(
            "MockCargoToml::update_dependency_version called for dep={} new_version={}",
            dep_name,
            new_version
        );
        if *self.dependency_update_changes_something() {
            info!("MockCargoToml: simulating a successful dependency version update");
            Ok(true)
        } else {
            warn!("MockCargoToml: no update was performed in this mock scenario");
            Ok(false)
        }
    }
}

#[async_trait]
impl GatherBinTargetNames for MockCargoToml {
    type Error = CargoTomlError;

    async fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error> {
        trace!("MockCargoToml::gather_bin_target_names called");
        // Return whatever is configured
        Ok(self.bin_target_names().clone())
    }
}

impl Versioned for MockCargoToml {
    type Error = CargoTomlError;

    fn version(&self) -> Result<semver::Version, Self::Error> {
        trace!("MockCargoToml::version called");
        // For demonstration, if either "valid_version_for_publishing" or "valid_version_for_integrity" is `true`,
        // we treat that as a valid version. Otherwise, we simulate an error.
        if *self.valid_version_for_publishing() || *self.valid_version_for_integrity() {
            let ver = semver::Version::parse("1.2.3").unwrap();
            info!("MockCargoToml: returning simulated version {}", ver);
            Ok(ver)
        } else {
            error!("MockCargoToml: simulating invalid version format error");
            Err(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: self.path().clone(),
                version: "mock_invalid_version".to_string(),
            })
        }
    }
}

impl CheckExistence for MockCargoToml {
    type Error = CargoTomlError;

    fn check_existence(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::check_existence called");
        if *self.file_exists() {
            info!("MockCargoToml: file_existence check => OK");
            Ok(())
        } else {
            error!("MockCargoToml: file_existence check => simulating file not found");
            Err(CargoTomlError::FileNotFound {
                missing_file: self.path().clone(),
            })
        }
    }
}

impl CheckRequiredFieldsForPublishing for MockCargoToml {
    type Error = CargoTomlError;

    fn check_required_fields_for_publishing(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::check_required_fields_for_publishing called");
        if *self.all_required_fields_for_publishing_present() {
            info!("MockCargoToml: all_required_fields_for_publishing => OK");
            Ok(())
        } else {
            error!("MockCargoToml: simulating missing required field for publishing");
            Err(CargoTomlError::MissingRequiredFieldForPublishing {
                cargo_toml_file: self.path().clone(),
                field: "mock_missing_field".to_string(),
            })
        }
    }
}

impl CheckVersionValidityForPublishing for MockCargoToml {
    type Error = CargoTomlError;

    fn check_version_validity_for_publishing(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::check_version_validity_for_publishing called");
        if *self.valid_version_for_publishing() {
            info!("MockCargoToml: version is considered valid for publishing");
            Ok(())
        } else {
            error!("MockCargoToml: simulating invalid version for publishing");
            Err(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: self.path().clone(),
                version: "mock_invalid_version_for_publishing".to_string(),
            })
        }
    }
}

impl CheckRequiredFieldsForIntegrity for MockCargoToml {
    type Error = CargoTomlError;

    fn check_required_fields_for_integrity(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::check_required_fields_for_integrity called");
        if *self.all_required_fields_for_integrity_present() {
            info!("MockCargoToml: all_required_fields_for_integrity => OK");
            Ok(())
        } else {
            error!("MockCargoToml: simulating missing required field for integrity");
            Err(CargoTomlError::MissingRequiredFieldForIntegrity {
                cargo_toml_file: self.path().clone(),
                field: "mock_missing_field_integrity".to_string(),
            })
        }
    }
}

impl CheckVersionValidityForIntegrity for MockCargoToml {
    type Error = CargoTomlError;

    fn check_version_validity_for_integrity(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::check_version_validity_for_integrity called");
        if *self.valid_version_for_integrity() {
            info!("MockCargoToml: version is considered valid for integrity");
            Ok(())
        } else {
            error!("MockCargoToml: simulating invalid version for integrity");
            Err(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: self.path().clone(),
                version: "mock_invalid_version_for_integrity".to_string(),
            })
        }
    }
}

impl GetPackageSection for MockCargoToml {
    type Error = CargoTomlError;

    fn get_package_section(&self) -> Result<&toml::Value, Self::Error> {
        trace!("MockCargoToml::get_package_section called");
        // We'll return a dummy static reference if the mock is set up
        // as if the package section is present, otherwise simulate an error.
        lazy_static!{
            static ref DUMMY_TOML: toml::Value = toml::Value::String("mock_package_section".to_string());
        }
        if *self.all_required_fields_for_integrity_present() || *self.all_required_fields_for_publishing_present() {
            info!("MockCargoToml: returning a dummy package section");
            Ok(&DUMMY_TOML)
        } else {
            error!("MockCargoToml: simulating missing package section");
            Err(CargoTomlError::MissingPackageSection {
                cargo_toml_file: self.path().clone(),
            })
        }
    }
}

impl GetPackageSectionMut for MockCargoToml {
    type Error = CargoTomlError;

    fn get_package_section_mut(&mut self) -> Result<&mut toml::Value, Self::Error> {
        trace!("MockCargoToml::get_package_section_mut called");
        // For simplicity, we just simulate that it's not supported in this mock
        // or that the package section is missing.
        error!("MockCargoToml: simulating missing or unsupported package section mut");
        Err(CargoTomlError::MissingPackageSection {
            cargo_toml_file: self.path().clone(),
        })
    }
}

impl IsValidVersion for MockCargoToml {
    fn is_valid_version(&self, version: &str) -> bool {
        trace!("MockCargoToml::is_valid_version called with version={}", version);
        semver::Version::parse(version).is_ok()
    }
}

impl ValidateIntegrity for MockCargoToml {
    type Error = CargoTomlError;

    fn validate_integrity(&self) -> Result<(), Self::Error> {
        trace!("MockCargoToml::validate_integrity called");
        self.check_existence()?;
        self.check_required_fields_for_integrity()?;
        self.check_version_validity_for_integrity()?;
        info!("MockCargoToml: integrity validation passed");
        Ok(())
    }
}

impl GetPackageAuthors for MockCargoToml {
    type Error = CargoTomlError;

    fn get_package_authors(&self) -> Result<Option<Vec<String>>, Self::Error> {
        trace!("MockCargoToml::get_package_authors called");
        Ok(self.package_authors().clone())
    }
}

#[async_trait]
impl GetPackageAuthorsOrFallback for MockCargoToml {
    type Error = CargoTomlError;

    async fn get_package_authors_or_fallback(&self) -> Result<Option<Vec<String>>, Self::Error> {
        trace!("MockCargoToml::get_package_authors_or_fallback called");
        if let Some(list) = self.package_authors() {
            info!("MockCargoToml: returning authors from mock");
            Ok(Some(list.clone()))
        } else {
            warn!("MockCargoToml: no authors found, returning None");
            Ok(None)
        }
    }
}

impl GetRustEdition for MockCargoToml {
    type Error = CargoTomlError;

    fn get_rust_edition(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_rust_edition called");
        Ok(self.rust_edition().clone())
    }
}

#[async_trait]
impl GetRustEditionOrFallback for MockCargoToml {
    type Error = CargoTomlError;

    async fn get_rust_edition_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_rust_edition_or_fallback called");
        if let Some(edition) = self.rust_edition() {
            info!("MockCargoToml: returning edition from mock");
            Ok(Some(edition.clone()))
        } else {
            warn!("MockCargoToml: no edition found, returning None");
            Ok(None)
        }
    }
}

impl GetLicenseType for MockCargoToml {
    type Error = CargoTomlError;

    fn get_license_type(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_license_type called");
        Ok(self.license().clone())
    }
}

#[async_trait]
impl GetLicenseTypeOrFallback for MockCargoToml {
    type Error = CargoTomlError;

    async fn get_license_type_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_license_type_or_fallback called");
        if let Some(lic) = self.license() {
            info!("MockCargoToml: returning license from mock");
            Ok(Some(lic.clone()))
        } else {
            warn!("MockCargoToml: no license found, returning None");
            Ok(None)
        }
    }
}

impl GetCrateRepositoryLocation for MockCargoToml {
    type Error = CargoTomlError;

    fn get_crate_repository_location(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_crate_repository_location called");
        Ok(self.repository().clone())
    }
}

#[async_trait]
impl GetCrateRepositoryLocationOrFallback for MockCargoToml {
    type Error = CargoTomlError;

    async fn get_crate_repository_location_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        trace!("MockCargoToml::get_crate_repository_location_or_fallback called");
        if let Some(repo) = self.repository() {
            info!("MockCargoToml: returning repository from mock");
            Ok(Some(repo.clone()))
        } else {
            warn!("MockCargoToml: no repository found, returning None");
            Ok(None)
        }
    }
}

/// We're implementing the big aggregator trait.
impl CargoTomlInterface for MockCargoToml {}

#[async_trait]
impl DocumentClone for MockCargoToml {
    type Error = CargoTomlError;
    async fn document_clone(&self) -> Result<toml_edit::Document,Self::Error> {
        todo!();
    }
}

#[async_trait]
impl WriteDocumentBack for MockCargoToml {
    type Error = CargoTomlError;
    async fn write_document_back(&mut self, doc: &toml_edit::Document) 
        -> Result<(),Self::Error>
    {
        todo!();
    }
}

impl MockCargoToml {
    /// Creates a MockCargoToml that is fully valid in every way.
    pub fn fully_valid_config() -> Self {
        trace!("MockCargoToml::fully_valid_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(true)
            .package_authors(Some(vec!["Alice <alice@example.com>".to_string()]))
            .rust_edition(Some("2021".to_string()))
            .license(Some("MIT".to_string()))
            .repository(Some("https://example.com/myrepo.git".to_string()))
            .bin_target_names(vec!["cli-tool".to_string()])
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that simulates a missing file (check_existence will fail).
    pub fn missing_file_config() -> Self {
        trace!("MockCargoToml::missing_file_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(false)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that is valid except it is missing required fields for publishing.
    pub fn missing_required_fields_for_publishing_config() -> Self {
        trace!("MockCargoToml::missing_required_fields_for_publishing_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(false)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that is valid except it is missing required fields for integrity.
    pub fn missing_required_fields_for_integrity_config() -> Self {
        trace!("MockCargoToml::missing_required_fields_for_integrity_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(false)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that is valid except for an invalid version for publishing.
    pub fn invalid_version_for_publishing_config() -> Self {
        trace!("MockCargoToml::invalid_version_for_publishing_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(false)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that is valid except for an invalid version for integrity.
    pub fn invalid_version_for_integrity_config() -> Self {
        trace!("MockCargoToml::invalid_version_for_integrity_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(false)
            .simulate_save_error(false)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }

    /// Creates a MockCargoToml that is valid except it always simulates an I/O error on save.
    pub fn simulate_save_error_config() -> Self {
        trace!("MockCargoToml::simulate_save_error_config constructor called");
        MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(true)
            .dependency_update_changes_something(false)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod test_mock_cargo_toml {
    use super::*;

    #[traced_test]
    fn test_mock_cargo_toml_basic() {
        let mock = MockCargoTomlBuilder::default()
            .path("fake/path/Cargo.toml")
            .file_exists(true)
            .all_required_fields_for_publishing_present(true)
            .all_required_fields_for_integrity_present(true)
            .valid_version_for_publishing(true)
            .valid_version_for_integrity(true)
            .simulate_save_error(false)
            .dependency_update_changes_something(true)
            .package_authors(Some(vec!["Alice <alice@example.com>".to_string()]))
            .rust_edition(Some("2021".to_string()))
            .license(Some("MIT".to_string()))
            .repository(Some("https://example.com/myrepo.git".to_string()))
            .bin_target_names(vec!["cli-tool".to_string()])
            .build()
            .unwrap();

        // Ensure existence check is OK
        let existence_result = mock.check_existence();
        assert!(existence_result.is_ok(), "Expected file to exist in mock");

        // Ensure version is valid
        let ver = mock.version().expect("Expected valid version");
        assert_eq!(ver.to_string(), "1.2.3");

        // Save to disk (should succeed because simulate_save_error is false)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let save_res = rt.block_on(mock.save_to_disk());
        assert!(save_res.is_ok(), "Expected save_to_disk to succeed");

        // Check authors
        let authors = mock.get_package_authors().expect("get_package_authors should not fail");
        assert_eq!(authors, Some(vec!["Alice <alice@example.com>".to_string()]));

        // Check bin target names
        let bins = mock.gather_bin_target_names().expect("gather_bin_target_names should not fail");
        assert_eq!(bins, vec!["cli-tool"]);

        // Check fallback authors (should be same as direct authors for this mock)
        let authors_fallback = rt.block_on(mock.get_package_authors_or_fallback()).unwrap();
        assert_eq!(authors_fallback, Some(vec!["Alice <alice@example.com>".to_string()]));
    }

    #[traced_test]
    fn test_mock_cargo_toml_fully_valid_config() {
        let mock = MockCargoToml::fully_valid_config();

        // All checks should pass
        assert!(mock.check_existence().is_ok());
        assert!(mock.check_required_fields_for_publishing().is_ok());
        assert!(mock.check_required_fields_for_integrity().is_ok());
        assert!(mock.check_version_validity_for_publishing().is_ok());
        assert!(mock.check_version_validity_for_integrity().is_ok());
        assert!(mock.validate_integrity().is_ok());

        let version = mock.version().expect("Expected valid version");
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[traced_test]
    fn test_mock_cargo_toml_missing_file_config() {
        let mock = MockCargoToml::missing_file_config();

        // Existence check should fail
        let existence_result = mock.check_existence();
        assert!(
            existence_result.is_err(),
            "Expected check_existence to fail for a missing file"
        );
    }

    #[traced_test]
    fn test_mock_cargo_toml_missing_required_fields_for_publishing_config() {
        let mock = MockCargoToml::missing_required_fields_for_publishing_config();

        // This should fail publishing field checks
        let fields_res = mock.check_required_fields_for_publishing();
        assert!(
            fields_res.is_err(),
            "Expected check_required_fields_for_publishing to fail"
        );
    }

    #[traced_test]
    fn test_mock_cargo_toml_missing_required_fields_for_integrity_config() {
        let mock = MockCargoToml::missing_required_fields_for_integrity_config();

        // This should fail integrity field checks
        let fields_res = mock.check_required_fields_for_integrity();
        assert!(
            fields_res.is_err(),
            "Expected check_required_fields_for_integrity to fail"
        );
    }

    #[traced_test]
    fn test_mock_cargo_toml_invalid_version_for_publishing_config() {
        let mock = MockCargoToml::invalid_version_for_publishing_config();

        // This should fail the publishing version check
        let version_res = mock.check_version_validity_for_publishing();
        assert!(version_res.is_err(), "Expected invalid version for publishing");
    }

    #[traced_test]
    fn test_mock_cargo_toml_invalid_version_for_integrity_config() {
        let mock = MockCargoToml::invalid_version_for_integrity_config();

        // This should fail the integrity version check
        let version_res = mock.check_version_validity_for_integrity();
        assert!(version_res.is_err(), "Expected invalid version for integrity");
    }

    #[traced_test]
    fn test_mock_cargo_toml_simulate_save_error_config() {
        let mock = MockCargoToml::simulate_save_error_config();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let save_res = rt.block_on(mock.save_to_disk());
        assert!(
            save_res.is_err(),
            "Expected save_to_disk to fail due to simulated I/O error"
        );
    }
}
