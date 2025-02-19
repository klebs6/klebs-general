// ---------------- [ File: workspacer-interface/src/cargo_toml_interface.rs ]
crate::ix!();

pub trait CargoTomlInterface
: CheckExistence<Error=CargoTomlError>
+ Send
+ Sync
+ PinWildcardDependencies<Error=CargoTomlError>
+ CheckRequiredFieldsForPublishing<Error=CargoTomlError>
+ CheckVersionValidityForPublishing<Error=CargoTomlError>
+ CheckRequiredFieldsForIntegrity<Error=CargoTomlError>
+ CheckVersionValidityForIntegrity<Error=CargoTomlError>
+ GetPackageSection<Error=CargoTomlError>
+ ReadyForCargoPublish<Error=CargoTomlError>
+ IsValidVersion
+ ValidateIntegrity<Error=CargoTomlError>
+ AsRef<Path>
{}

pub type LockVersionMap = BTreeMap<String, BTreeSet<semver::Version>>;

#[async_trait]
pub trait PinWildcardDependencies {

    type Error;

    async fn pin_wildcard_dependencies(
        &self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), Self::Error>;
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

pub trait IsValidVersion {

    /// Checks if the version string is a valid SemVer version
    fn is_valid_version(&self, version: &str) -> bool;
}
