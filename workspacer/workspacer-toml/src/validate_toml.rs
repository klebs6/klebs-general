// ---------------- [ File: src/validate_toml.rs ]
crate::ix!();

/// Validates the provided TOML data, ensuring that a `[package]` section
/// is present and its fields meet certain criteria (e.g. valid SemVer version,
/// mandatory fields like `authors`, `license`, etc.).
///
/// Returns `Ok(())` if everything is valid, or a `CargoTomlError` variant
/// indicating the problem otherwise.
pub fn validate_toml(toml_value: &toml::Value) -> Result<(), CargoTomlError> {
    // 1) Ensure there is a [package] section
    let package = toml_value.get("package").ok_or_else(|| CargoTomlError::MissingPackageSection {
        cargo_toml_file: PathBuf::from("Cargo.toml"),
    })?;

    // 2) Check that "version" is present and valid
    if let Some(version_str) = package.get("version").and_then(|v| v.as_str()) {
        if semver::Version::parse(version_str).is_err() {
            return Err(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: PathBuf::from("Cargo.toml"),
                version: version_str.to_string(),
            });
        }
    } else {
        // If you want to strictly require "version":
        return Err(CargoTomlError::MissingRequiredFieldForPublishing {
            cargo_toml_file: PathBuf::from("Cargo.toml"),
            field: "version".to_string(),
        });
    }

    // 3) Optionally check other fields that might be required for publishing
    // For example: name, authors, license
    if package.get("name").and_then(|n| n.as_str()).is_none() {
        return Err(CargoTomlError::MissingRequiredFieldForPublishing {
            cargo_toml_file: PathBuf::from("Cargo.toml"),
            field: "name".to_string(),
        });
    }

    // authors is often an array of strings
    let authors_present = package
        .get("authors")
        .and_then(|a| a.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    if !authors_present {
        return Err(CargoTomlError::MissingRequiredFieldForPublishing {
            cargo_toml_file: PathBuf::from("Cargo.toml"),
            field: "authors".to_string(),
        });
    }

    // license is often a single string
    if package.get("license").and_then(|l| l.as_str()).is_none() {
        return Err(CargoTomlError::MissingRequiredFieldForPublishing {
            cargo_toml_file: PathBuf::from("Cargo.toml"),
            field: "license".to_string(),
        });
    }

    // 4) If we reach here, everything we require is valid
    Ok(())
}

// ---------------------------------------------------------------------
// Test Module for `validate_toml`
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests_validate_toml {
    use super::*;
    use toml::Value as TomlValue;

    #[test]
    fn test_validate_toml_success_when_all_fields_valid() {
        let toml_str = r#"
            [package]
            name = "my_crate"
            version = "1.2.3"
            authors = ["Somebody <somebody@example.com>"]
            license = "MIT"
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
    }

    #[test]
    fn test_validate_toml_error_when_package_section_missing() {
        let toml_str = r#"
            # no [package] table at all
            some_other_table = { }
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::MissingPackageSection { .. }) => { /* correct */ }
            other => panic!("Expected MissingPackageSection error, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_toml_error_when_version_missing() {
        let toml_str = r#"
            [package]
            name = "my_crate"
            authors = ["Author <author@example.com>"]
            license = "MIT"
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. })
                if field == "version" => { /* correct */ }
            other => panic!("Expected MissingRequiredFieldForPublishing for 'version', got {:?}", other),
        }
    }

    #[test]
    fn test_validate_toml_error_when_version_is_invalid_semver() {
        let toml_str = r#"
            [package]
            name = "my_crate"
            version = "abc.def"
            authors = ["Someone <someone@example.com>"]
            license = "MIT"
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::InvalidVersionFormat { version, .. })
                if version == "abc.def" => { /* correct */ }
            other => panic!("Expected InvalidVersionFormat error, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_toml_error_when_name_missing() {
        let toml_str = r#"
            [package]
            version = "1.0.0"
            authors = ["Author <author@example.com>"]
            license = "MIT"
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. })
                if field == "name" => { /* correct */ }
            other => panic!("Expected MissingRequiredFieldForPublishing for 'name', got {:?}", other),
        }
    }

    #[test]
    fn test_validate_toml_error_when_authors_missing() {
        let toml_str = r#"
            [package]
            name = "my_crate"
            version = "1.0.0"
            license = "MIT"
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. })
                if field == "authors" => { /* correct */ }
            other => panic!("Expected MissingRequiredFieldForPublishing for 'authors', got {:?}", other),
        }
    }

    #[test]
    fn test_validate_toml_error_when_license_missing() {
        let toml_str = r#"
            [package]
            name = "my_crate"
            version = "1.0.0"
            authors = ["Author <author@example.com>"]
        "#;
        let toml_value: TomlValue = toml::from_str(toml_str).unwrap();

        let result = validate_toml(&toml_value);
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. })
                if field == "license" => { /* correct */ }
            other => panic!("Expected MissingRequiredFieldForPublishing for 'license', got {:?}", other),
        }
    }
}
