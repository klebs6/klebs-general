// ---------------- [ File: workspacer-toml/src/check_version_validity_for_publishing.rs ]
crate::ix!();

impl CheckVersionValidityForPublishing for CargoToml {

    type Error = CargoTomlError;

    /// Ensures that the version field is valid
    fn check_version_validity_for_publishing(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            if !self.is_valid_version(version) {
                return Err(CargoTomlError::InvalidVersionFormat {
                    cargo_toml_file: self.path().clone(),
                    version: version.to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_check_version_validity_for_publishing {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::fs;
    
    

    /// Helper function: writes `contents` to a temporary file, then
    /// calls `CargoToml::new` to return a `CargoToml` handle.
    async fn create_cargo_toml_file(contents: &str) -> Result<CargoToml, CargoTomlError> {
        // Create a temporary file
        let mut temp = NamedTempFile::new().expect("failed to create temp file");
        write!(temp, "{}", contents).expect("failed to write to temp file");
        let path = temp.into_temp_path();

        // Convert to a path we can read from asynchronously
        let path_buf = path.to_path_buf();
        fs::write(&path_buf, contents)
            .await
            .expect("failed to write to temp file async");

        // Build the CargoToml handle
        CargoToml::new(&path_buf).await
    }

    /// 1) No `[package]` section at all.
    ///    -> Should fail with `MissingPackageSection` (from `get_package_section`).
    #[tokio::test]
    async fn test_no_package_section() {
        let toml_str = r#"
            [dependencies]
            serde = "1.0"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Could not create test CargoToml");
        
        let result = cargo_toml.check_version_validity_for_publishing();
        match result {
            Ok(_) => panic!("Expected an error due to missing [package]"),
            Err(e) => match e {
                CargoTomlError::MissingPackageSection { .. } => {
                    // correct error variant
                },
                _ => panic!("Unexpected error variant: {:?}", e),
            },
        }
    }

    /// 2) `[package]` is present, but no `version` field at all.
    ///    -> Should succeed because we only fail if a version is present but invalid.
    #[tokio::test]
    async fn test_package_but_no_version() {
        let toml_str = r#"
            [package]
            name = "my-crate"
            # no version
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Could not create test CargoToml");
        
        // Should succeed, because no version -> no check needed
        cargo_toml
            .check_version_validity_for_publishing()
            .expect("Expected success when version is absent");
    }

    /// 3) `[package]` has a valid SemVer version
    ///    -> Should succeed with no errors.
    #[tokio::test]
    async fn test_valid_semver_version() {
        let toml_str = r#"
            [package]
            name = "my-crate"
            version = "1.2.3-alpha.1"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Could not create test CargoToml");
        
        cargo_toml
            .check_version_validity_for_publishing()
            .expect("Expected success for valid semver");
    }

    /// 4) `[package].version` is invalid SemVer (like "not-semver")
    ///    -> Should fail with `InvalidVersionFormat`.
    #[tokio::test]
    async fn test_invalid_semver_version() {
        let toml_str = r#"
            [package]
            name = "my-crate"
            version = "not-semver"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Could not create test CargoToml");
        
        let result = cargo_toml.check_version_validity_for_publishing();
        match result {
            Ok(_) => panic!("Expected error due to invalid semver in version"),
            Err(e) => match e {
                CargoTomlError::InvalidVersionFormat { version, .. } => {
                    assert_eq!(version, "not-semver");
                },
                _ => panic!("Unexpected error variant: {:?}", e),
            },
        }
    }

    /// 5) `[package].version` is present but not a string (e.g. `version = 123`).
    ///    -> Then `package.get("version").and_then(|v| v.as_str())` is `None`.
    ///    -> So no error is triggered, effectively ignoring the version.
    #[tokio::test]
    async fn test_version_is_non_string_type() {
        let toml_str = r#"
            [package]
            name = "my-crate"
            version = 123
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Could not create test CargoToml");

        cargo_toml
            .check_version_validity_for_publishing()
            .expect("Expected success when version is not a string");
    }

    #[test]
    fn check_version_validity_for_publishing_succeeds_with_valid_version() {
        // Provide a valid version in the package section
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("valid_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.2.3".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_version_validity_for_publishing();
        assert!(
            result.is_ok(),
            "Expected Ok for a valid semver version (1.2.3)"
        );
    }

    #[test]
    fn check_version_validity_for_publishing_fails_when_version_is_invalid() {
        // Provide an invalid version string in the package section
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("invalid_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("not.semver".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_version_validity_for_publishing();
        match result {
            Err(CargoTomlError::InvalidVersionFormat { version, .. }) => {
                assert_eq!(version, "not.semver");
            }
            other => {
                panic!("Expected InvalidVersionFormat error for invalid semver; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_version_validity_for_publishing_succeeds_when_version_is_missing() {
        // If version is entirely missing, it won't fail here
        // because check_version_validity_for_publishing only fails on an *invalid* version string.
        // (If "version" is missing, a different trait likely catches that.)
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        // NO version field here

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_version_validity_for_publishing();
        assert!(result.is_ok(), "Expected Ok because the version is simply not present");
    }

    #[test]
    fn check_version_validity_for_publishing_fails_when_package_section_is_missing() {
        // If [package] is missing, get_package_section() should fail
        let root_map = toml::map::Map::new();

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_version_validity_for_publishing();
        match result {
            Err(CargoTomlError::MissingPackageSection { .. }) => {
                // Good
            }
            other => {
                panic!("Expected MissingPackageSection error; got {:?}", other);
            }
        }
    }
}
