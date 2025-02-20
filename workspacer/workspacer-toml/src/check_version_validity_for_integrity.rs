// ---------------- [ File: src/check_version_validity_for_integrity.rs ]
crate::ix!();

impl CheckVersionValidityForIntegrity for CargoToml {
    type Error = CargoTomlError;

    /// Ensures that the version field is valid for integrity purposes.
    ///
    /// - If `[package]` is missing, returns `CargoTomlError::MissingPackageSection`.
    /// - If `version` is present but invalid, returns `CargoTomlError::InvalidVersionFormat`.
    /// - If `version` is absent or non-string, it silently passes (OK).
    fn check_version_validity_for_integrity(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?; // might return MissingPackageSection
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
mod tests_check_version_validity_for_integrity {
    use super::*;
    use toml::from_str;

    /// Helper to build a `CargoToml` purely in-memory from a &str.
    /// In real tests, you might create a tempfile or a mocked struct instead.
    fn make_cargo_toml_from_str(toml_data: &str) -> CargoToml {
        let parsed: TomlValue = from_str(toml_data).unwrap();
        CargoTomlBuilder::default()
            .path(PathBuf::from("Cargo.toml"))
            .content(parsed)
            .build()
            .unwrap()
    }

    #[test]
    fn test_check_version_validity_for_integrity_ok_with_valid_version() {
        let toml_data = r#"
            [package]
            name = "my_crate"
            version = "1.2.3"
        "#;
        let cargo_toml = make_cargo_toml_from_str(toml_data);

        let result = cargo_toml.check_version_validity_for_integrity();
        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
    }

    #[test]
    fn test_check_version_validity_for_integrity_ok_when_version_missing() {
        // The current implementation treats a missing version as OK
        // for "integrity" (it doesn't require the version to be present).
        let toml_data = r#"
            [package]
            name = "my_crate"
            # version is omitted
        "#;
        let cargo_toml = make_cargo_toml_from_str(toml_data);

        let result = cargo_toml.check_version_validity_for_integrity();
        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
    }

    #[test]
    fn test_check_version_validity_for_integrity_ok_when_package_has_non_string_version() {
        // If version is not a string, `and_then(|v| v.as_str())` will return None,
        // effectively skipping the check and returning Ok(()). Adjust if desired.
        let toml_data = r#"
            [package]
            name = "my_crate"
            version = 123  # not a string
        "#;
        let cargo_toml = make_cargo_toml_from_str(toml_data);

        let result = cargo_toml.check_version_validity_for_integrity();
        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
    }

    #[test]
    fn test_check_version_validity_for_integrity_err_when_invalid_semver() {
        let toml_data = r#"
            [package]
            name = "my_crate"
            version = "not_semver"
        "#;
        let cargo_toml = make_cargo_toml_from_str(toml_data);

        let result = cargo_toml.check_version_validity_for_integrity();
        match result {
            Err(CargoTomlError::InvalidVersionFormat { version, .. })
                if version == "not_semver" => { /* correct */ }
            other => panic!("Expected InvalidVersionFormat, got {:?}", other),
        }
    }

    #[test]
    fn test_check_version_validity_for_integrity_err_when_package_missing() {
        // If `[package]` table is missing, `get_package_section()` should error out.
        let toml_data = r#"
            [not_package]
            version = "1.2.3"
        "#;
        let cargo_toml = make_cargo_toml_from_str(toml_data);

        let result = cargo_toml.check_version_validity_for_integrity();
        match result {
            Err(CargoTomlError::MissingPackageSection { .. }) => { /* correct */ }
            other => panic!("Expected MissingPackageSection, got {:?}", other),
        }
    }
}
