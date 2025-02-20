// ---------------- [ File: src/validate_integrity.rs ]
crate::ix!();

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

#[cfg(test)]
mod test_validate_integrity {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs::File;

    #[test]
    fn validate_integrity_succeeds_with_valid_file_and_fields() {
        // Create a temp dir and a valid Cargo.toml file
        let temp = tempdir().expect("Failed to create temp directory for test");
        let file_path = temp.path().join("Cargo.toml");

        // Actually create the file so check_existence() passes
        File::create(&file_path).expect("Failed to create a test file");

        // Build minimal valid "package" content for check_required_fields_for_integrity
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("0.1.0".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        // Also ensure version is valid for check_version_validity_for_integrity
        let result = cargo_toml.validate_integrity();
        assert!(result.is_ok(), "Expected Ok for valid file, presence of fields, and valid version");
    }

    #[test]
    fn validate_integrity_fails_when_file_does_not_exist() {
        // Provide a path that doesn't exist
        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("non_existent_dir/non_existent.toml"))
            .content(toml::Value::Table(toml::map::Map::new()))
            .build()
            .unwrap();

        let result = cargo_toml.validate_integrity();
        match result {
            Err(CargoTomlError::FileNotFound { .. }) => {
                // Good
            }
            other => {
                panic!("Expected FileNotFound error; got {:?}", other);
            }
        }
    }

    #[test]
    fn validate_integrity_fails_when_required_fields_for_integrity_are_missing() {
        // Create a temp file but with incomplete package data
        let temp = tempdir().expect("Failed to create temp directory for test");
        let file_path = temp.path().join("Cargo.toml");
        File::create(&file_path).expect("Failed to create test file");

        // "package" has no "version" field
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.validate_integrity();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForIntegrity { field, .. }) => {
                assert_eq!(field, "version");
            }
            other => {
                panic!("Expected MissingRequiredFieldForIntegrity error for 'version'; got {:?}", other);
            }
        }
    }

    #[test]
    fn validate_integrity_fails_on_invalid_version() {
        // Create a temp file but give it an invalid semver
        let temp = tempdir().expect("Failed to create temp directory for test");
        let file_path = temp.path().join("Cargo.toml");
        File::create(&file_path).expect("Failed to create test file");

        // Provide a package table with an invalid version
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("abcxyz".into())); // invalid semver

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.validate_integrity();
        match result {
            Err(CargoTomlError::InvalidVersionFormat { version, .. }) => {
                assert_eq!(version, "abcxyz");
            }
            other => {
                panic!("Expected InvalidVersionFormat error for invalid version string; got {:?}", other);
            }
        }
    }
}
