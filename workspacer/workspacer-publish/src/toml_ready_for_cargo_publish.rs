// ---------------- [ File: src/ready_for_cargo_publish.rs ]
crate::ix!();

#[async_trait]
impl ReadyForCargoPublish for dyn CargoTomlInterface {

    type Error = CargoTomlError;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {
        self.validate_integrity()?;
        self.check_required_fields_for_publishing()?;
        self.check_version_validity_for_publishing()?;
        Ok(())
    }
}

#[cfg(test)]
#[disable]
mod test_ready_for_cargo_publish {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs::File;

    #[tokio::test]
    async fn ready_for_cargo_publish_succeeds_when_all_checks_pass() {
        // 1) The file must exist
        // 2) Required fields for integrity and publishing must exist
        // 3) Semver version must be valid

        let temp = tempdir().expect("Failed to create temp directory");
        let file_path = temp.path().join("Cargo.toml");

        // Actually create the file so check_existence() passes
        File::create(&file_path).expect("Failed to create a test Cargo.toml file");

        // Provide minimal "package" data for *both* integrity and publishing
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("publishable_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.0.0".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        package_table.insert("license".to_string(), toml::Value::String("MIT".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        // Build the CargoToml
        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "Expected success because all checks should pass");
    }

    #[tokio::test]
    async fn ready_for_cargo_publish_fails_when_file_does_not_exist() {
        // This should fail on check_existence()
        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("non_existent_location/Cargo.toml"))
            .content(toml::Value::Table(toml::map::Map::new()))
            .build()
            .unwrap();

        let result = cargo_toml.ready_for_cargo_publish().await;
        match result {
            Err(CargoTomlError::FileNotFound { .. }) => {
                // good
            }
            other => {
                panic!("Expected FileNotFound error; got {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn ready_for_cargo_publish_fails_when_missing_fields_for_integrity() {
        // "name" is present, but "version" is missing => fails on check_required_fields_for_integrity()
        let temp = tempdir().expect("Failed to create temp directory");
        let file_path = temp.path().join("Cargo.toml");
        File::create(&file_path).expect("Failed to create test file");

        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("crate_without_version".into()));
        // no "version"

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.ready_for_cargo_publish().await;
        match result {
            Err(CargoTomlError::MissingRequiredFieldForIntegrity { field, .. }) => {
                assert_eq!(field, "version");
            }
            other => {
                panic!("Expected MissingRequiredFieldForIntegrity for 'version'; got {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn ready_for_cargo_publish_fails_when_missing_fields_for_publishing() {
        // We'll have "name" and "version" for integrity, but omit "authors" or "license",
        // which should fail check_required_fields_for_publishing()

        let temp = tempdir().expect("Failed to create temp directory");
        let file_path = temp.path().join("Cargo.toml");
        File::create(&file_path).expect("Failed to create test file");

        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("crate_for_publish_test".into()));
        package_table.insert("version".to_string(), toml::Value::String("0.5.0".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        // no "license" field

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.ready_for_cargo_publish().await;
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. }) => {
                assert_eq!(field, "license");
            }
            other => {
                panic!("Expected MissingRequiredFieldForPublishing for 'license'; got {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn ready_for_cargo_publish_fails_when_version_is_invalid() {
        // "validate_integrity()" can pass if there's a valid file with minimal fields,
        // but "check_version_validity_for_publishing()" fails if version is semver-invalid.

        let temp = tempdir().expect("Failed to create temp directory");
        let file_path = temp.path().join("Cargo.toml");
        File::create(&file_path).expect("Failed to create test file");

        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("invalid_version_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("xyz.notsemver".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        package_table.insert("license".to_string(), toml::Value::String("Apache-2.0".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(file_path)
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.ready_for_cargo_publish().await;
        match result {
            Err(CargoTomlError::InvalidVersionFormat { version, .. }) => {
                assert_eq!(version, "xyz.notsemver");
            }
            other => {
                panic!("Expected InvalidVersionFormat error; got {:?}", other);
            }
        }
    }
}
