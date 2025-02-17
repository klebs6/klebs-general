crate::ix!();

impl CheckRequiredFieldsForPublishing for CargoToml {

    type Error = CargoTomlError;

    /// Checks if `Cargo.toml` has required fields for publishing
    fn check_required_fields_for_publishing(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;

        let required_fields = ["name", "version", "authors", "license"];
        for field in &required_fields {
            if package.get(field).is_none() {
                return Err(CargoTomlError::MissingRequiredFieldForPublishing {
                    cargo_toml_file: self.path().clone(),
                    field: field.to_string(),
                });
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod test_check_required_fields_for_publishing {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn check_required_fields_for_publishing_succeeds_with_all_required_fields() {
        // Create a minimal "package" table with all required fields: name, version, authors, license
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.0.0".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        package_table.insert("license".to_string(), toml::Value::String("MIT".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        // Manually build CargoToml with the above content
        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        assert!(
            result.is_ok(),
            "Expected Ok because all required fields (name, version, authors, license) are present"
        );
    }

    #[test]
    fn check_required_fields_for_publishing_fails_when_name_is_missing() {
        // Provide version, authors, license but no "name"
        let mut package_table = toml::map::Map::new();
        package_table.insert("version".to_string(), toml::Value::String("1.0.0".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        package_table.insert("license".to_string(), toml::Value::String("MIT".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. }) => {
                assert_eq!(field, "name");
            }
            other => {
                panic!("Expected MissingRequiredFieldForPublishing for 'name'; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_required_fields_for_publishing_fails_when_version_is_missing() {
        // Provide name, authors, license but no "version"
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));
        package_table.insert("license".to_string(), toml::Value::String("MIT".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. }) => {
                assert_eq!(field, "version");
            }
            other => {
                panic!("Expected MissingRequiredFieldForPublishing for 'version'; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_required_fields_for_publishing_fails_when_authors_is_missing() {
        // Provide name, version, license but no "authors"
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.0.0".into()));
        package_table.insert("license".to_string(), toml::Value::String("MIT".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. }) => {
                assert_eq!(field, "authors");
            }
            other => {
                panic!("Expected MissingRequiredFieldForPublishing for 'authors'; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_required_fields_for_publishing_fails_when_license_is_missing() {
        // Provide name, version, authors but no "license"
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.0.0".into()));
        package_table.insert("authors".to_string(), toml::Value::String("Someone <someone@example.com>".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForPublishing { field, .. }) => {
                assert_eq!(field, "license");
            }
            other => {
                panic!("Expected MissingRequiredFieldForPublishing for 'license'; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_required_fields_for_publishing_fails_when_package_section_is_missing() {
        // Provide no [package] at all
        let root_map = toml::map::Map::new();

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("some/fake/path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_publishing();
        match result {
            Err(CargoTomlError::MissingPackageSection { .. }) => {
                // This is what we expect
            }
            other => {
                panic!("Expected MissingPackageSection error; got {:?}", other);
            }
        }
    }
}
