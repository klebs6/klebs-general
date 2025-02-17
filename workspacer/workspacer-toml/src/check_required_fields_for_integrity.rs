crate::ix!();

impl CheckRequiredFieldsForIntegrity for CargoToml {

    type Error = CargoTomlError;

    /// Checks if `Cargo.toml` has required fields for integrity purposes
    fn check_required_fields_for_integrity(&self) -> Result<(), Self::Error> {
        let package = self.get_package_section()?;

        let required_fields = ["name", "version"];
        for field in &required_fields {
            if package.get(field).is_none() {
                return Err(CargoTomlError::MissingRequiredFieldForIntegrity {
                    cargo_toml_file: self.path().clone(),
                    field: field.to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_check_required_fields_for_integrity {
    use super::*;  // bring `CargoToml`, `CheckRequiredFieldsForIntegrity`, etc. into scope
    use std::path::PathBuf;

    #[test]
    fn check_required_fields_for_integrity_ok_with_name_and_version() {
        // A minimal "package" table with "name" and "version".
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));
        package_table.insert("version".to_string(), toml::Value::String("1.2.3".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake_path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_integrity();
        assert!(result.is_ok(), "Expected OK when name and version are present");
    }

    #[test]
    fn check_required_fields_for_integrity_fails_if_name_missing() {
        // Provide version only
        let mut package_table = toml::map::Map::new();
        package_table.insert("version".to_string(), toml::Value::String("0.1.0".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake_path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_integrity();
        match result {
            Err(CargoTomlError::MissingRequiredFieldForIntegrity { field, .. }) => {
                assert_eq!(field, "name");
            }
            other => {
                panic!("Expected MissingRequiredFieldForIntegrity error for 'name'; got {:?}", other);
            }
        }
    }

    #[test]
    fn check_required_fields_for_integrity_fails_if_version_missing() {
        // Provide name only
        let mut package_table = toml::map::Map::new();
        package_table.insert("name".to_string(), toml::Value::String("demo_crate".into()));

        let mut root_map = toml::map::Map::new();
        root_map.insert("package".to_string(), toml::Value::Table(package_table));

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake_path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_integrity();
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
    fn check_required_fields_for_integrity_fails_if_package_section_missing() {
        // No [package] at all
        let root_map = toml::map::Map::new();

        let cargo_toml = CargoTomlBuilder::default()
            .path(PathBuf::from("fake_path/Cargo.toml"))
            .content(toml::Value::Table(root_map))
            .build()
            .unwrap();

        let result = cargo_toml.check_required_fields_for_integrity();
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
