crate::ix!();

/// Handle to manipulate and verify the `Cargo.toml` file
#[derive(Debug,Clone)]
pub struct CargoToml {
    path:    PathBuf,
    content: toml::Value,  // Parsed TOML content
}

impl CargoTomlInterface for CargoToml {}

impl CargoToml {

    /// Creates a new handle from the path to `Cargo.toml`
    pub async fn new<P>(cargo_toml_path: P) -> Result<Self, CargoTomlError> 
        where P: AsRef<Path>
    {
        let cargo_content = fs::read_to_string(&cargo_toml_path).await
            .map_err(|e| CargoTomlError::ReadError { io: e.into() })?;

        let parsed: toml::Value = toml::from_str(&cargo_content).map_err(|toml_parse_error| {
            CargoTomlError::TomlParseError {
                cargo_toml_file: cargo_toml_path.as_ref().to_path_buf(),
                toml_parse_error,
            }
        })?;

        Ok(Self {
            path: cargo_toml_path.as_ref().to_path_buf(),
            content: parsed,
        })
    }
}

impl AsRef<Path> for CargoToml {
    /// Allows `CargoToml` to be treated as a path
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl GetPackageSection for CargoToml {

    type Error = CargoTomlError;

    /// Helper to retrieve the `package` section from `Cargo.toml`
    fn get_package_section(&self) -> Result<&toml::Value, Self::Error> {
        self.content.get("package").ok_or_else(|| CargoTomlError::MissingPackageSection {
            cargo_toml_file: self.path.clone(),
        })
    }
}

impl IsValidVersion for CargoToml {

    /// Checks if the version string is a valid SemVer version
    fn is_valid_version(version: &str) -> bool {
        semver::Version::parse(version).is_ok()
    }
}

impl CheckExistence for CargoToml {

    type Error = CargoTomlError;

    fn check_existence(&self) -> Result<(), Self::Error> {
        if !self.path.exists() {
            return Err(CargoTomlError::FileNotFound {
                missing_file: self.path.clone()
            });
        }
        Ok(())
    }
}

impl CheckRequiredFieldsForPublishing for CargoToml {

    type Error = CargoTomlError;

    /// Checks if `Cargo.toml` has required fields for publishing
    fn check_required_fields_for_publishing(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;

        let required_fields = ["name", "version", "authors", "license"];
        for field in &required_fields {
            if package.get(field).is_none() {
                return Err(CargoTomlError::MissingRequiredFieldForPublishing {
                    cargo_toml_file: self.path.clone(),
                    field: field.to_string(),
                });
            }
        }

        Ok(())
    }
}

impl CheckVersionValidityForPublishing for CargoToml {

    type Error = CargoTomlError;

    /// Ensures that the version field is valid
    fn check_version_validity_for_publishing(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            if !Self::is_valid_version(version) {
                return Err(CargoTomlError::InvalidVersionFormat {
                    cargo_toml_file: self.path.clone(),
                    version: version.to_string(),
                });
            }
        }

        Ok(())
    }
}

impl CheckRequiredFieldsForIntegrity for CargoToml {

    type Error = CargoTomlError;

    /// Checks if `Cargo.toml` has required fields for integrity purposes
    fn check_required_fields_for_integrity(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;

        let required_fields = ["name", "version"];
        for field in &required_fields {
            if package.get(field).is_none() {
                return Err(CargoTomlError::MissingRequiredFieldForIntegrity {
                    cargo_toml_file: self.path.clone(),
                    field: field.to_string(),
                });
            }
        }

        Ok(())
    }
}

impl CheckVersionValidityForIntegrity for CargoToml {

    type Error = CargoTomlError;

    /// Ensures that the version field is valid for integrity purposes
    fn check_version_validity_for_integrity(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            if !Self::is_valid_version(version) {
                return Err(CargoTomlError::InvalidVersionFormat {
                    cargo_toml_file: self.path.clone(),
                    version: version.to_string(),
                });
            }
        }
        Ok(())
    }
}

impl ValidateIntegrity for CargoToml {

    type Error = CargoTomlError;

    /// Validates the integrity of a crate by checking required files and directory structure
    fn validate_integrity(&self) -> Result<(), Self::Error> {
        self.check_existence()?;
        self.check_required_fields_for_integrity()?;
        self.check_version_validity_for_integrity()?;
        Ok(())
    }
}

#[async_trait]
impl ReadyForCargoPublish for CargoToml {

    type Error = CargoTomlError;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {
        self.validate_integrity()?;
        self.check_required_fields_for_publishing()?;
        self.check_version_validity_for_publishing()?;
        Ok(())
    }
}
