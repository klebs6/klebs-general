// ---------------- [ File: workspacer-toml-interface/src/interface.rs ]
crate::ix!();

pub trait CargoTomlInterface
: CheckExistence<Error=CargoTomlError>
+ Send
+ Sync
+ Versioned<Error=CargoTomlError>
//+ PinWildcardDependencies<Error=CargoTomlError>
+ CheckRequiredFieldsForPublishing<Error=CargoTomlError>
+ CheckVersionValidityForPublishing<Error=CargoTomlError>
+ CheckRequiredFieldsForIntegrity<Error=CargoTomlError>
+ CheckVersionValidityForIntegrity<Error=CargoTomlError>
+ SaveToDisk<Error=CargoTomlError>
+ UpdateDependencyVersionRaw<Error=CargoTomlError>
+ GetPackageSection<Error=CargoTomlError>
+ GetPackageSectionMut<Error=CargoTomlError>
//+ ReadyForCargoPublish<Error=CargoTomlError>
+ IsValidVersion
+ ValidateIntegrity<Error=CargoTomlError>
+ GatherBinTargetNames<Error=CargoTomlError>
+ AsRef<Path>
+ GetPackageAuthors<Error=CargoTomlError>
+ GetPackageAuthorsOrFallback<Error=CargoTomlError>
+ GetRustEdition<Error=CargoTomlError>
+ GetRustEditionOrFallback<Error=CargoTomlError>
+ GetLicenseType<Error=CargoTomlError>
+ GetLicenseTypeOrFallback<Error=CargoTomlError>
+ GetCrateRepositoryLocation<Error=CargoTomlError>
+ GetCrateRepositoryLocationOrFallback<Error=CargoTomlError>
{}

#[async_trait]
pub trait SaveToDisk {
    type Error;
    async fn save_to_disk(&self) -> Result<(), Self::Error>;
}

pub trait UpdateDependencyVersionRaw {
    type Error;
    fn update_dependency_version(
        &mut self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<bool, Self::Error>;
}

pub trait GatherBinTargetNames {
    type Error;
    fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error>;
}

pub trait Versioned {
    type Error: std::fmt::Debug;
    fn version(&self) -> Result<semver::Version,Self::Error>;
}

pub trait CheckExistence {

    type Error;

    fn check_existence(&self) -> Result<(), Self::Error>;
}

pub trait CheckRequiredFieldsForPublishing {

    type Error;

    /// Checks if `Cargo.toml` has required fields for publishing
    fn check_required_fields_for_publishing(&self) -> Result<(), Self::Error>;
}

pub trait CheckVersionValidityForPublishing {

    type Error;

    /// Ensures that the version field is valid
    fn check_version_validity_for_publishing(&self) -> Result<(), Self::Error>;
}

pub trait CheckRequiredFieldsForIntegrity {

    type Error;

    /// Checks if `Cargo.toml` has required fields for integrity purposes
    fn check_required_fields_for_integrity(&self) -> Result<(), Self::Error>;
}

pub trait CheckVersionValidityForIntegrity {

    type Error;

    /// Ensures that the version field is valid for integrity purposes
    fn check_version_validity_for_integrity(&self) -> Result<(), Self::Error>;
}

pub trait GetPackageSection {

    type Error;

    /// Helper to retrieve the `package` section from `Cargo.toml`
    fn get_package_section(&self) -> Result<&toml::Value, Self::Error>;
}

pub trait GetPackageSectionMut {

    type Error;

    /// Helper to retrieve the `package` section from `Cargo.toml`
    fn get_package_section_mut(&mut self) -> Result<&mut toml::Value, Self::Error>;
}

pub trait IsValidVersion {

    /// Checks if the version string is a valid SemVer version
    fn is_valid_version(&self, version: &str) -> bool;
}

pub trait GetPackageAuthors {
    type Error;
    fn get_package_authors(&self) -> Result<Option<Vec<String>>, Self::Error>;
}

#[async_trait]
pub trait GetPackageAuthorsOrFallback {
    type Error;
    async fn get_package_authors_or_fallback(&self) -> Result<Option<Vec<String>>, Self::Error>;
}

pub trait GetRustEdition {
    type Error;
    fn get_rust_edition(&self) -> Result<Option<String>, Self::Error>;
}

#[async_trait]
pub trait GetRustEditionOrFallback {
    type Error;
    async fn get_rust_edition_or_fallback(&self) -> Result<Option<String>, Self::Error>;
}

pub trait GetLicenseType {
    type Error;
    fn get_license_type(&self) -> Result<Option<String>, Self::Error>;
}

#[async_trait]
pub trait GetLicenseTypeOrFallback {
    type Error;
    async fn get_license_type_or_fallback(&self) -> Result<Option<String>, Self::Error>;
}

pub trait GetCrateRepositoryLocation {
    type Error;
    fn get_crate_repository_location(&self) -> Result<Option<String>, Self::Error>;
}

#[async_trait]
pub trait GetCrateRepositoryLocationOrFallback {
    type Error;
    async fn get_crate_repository_location_or_fallback(&self) -> Result<Option<String>, Self::Error>;
}
