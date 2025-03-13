// ---------------- [ File: workspacer-toml/src/get_package_section.rs ]
crate::ix!();

impl GetPackageSection for CargoToml {

    type Error = CargoTomlError;

    /// Helper to retrieve the `package` section from `Cargo.toml`
    fn get_package_section(&self) -> Result<&toml::Value, Self::Error> {
        self.content().get("package").ok_or_else(|| CargoTomlError::MissingPackageSection {
            cargo_toml_file: self.path().clone(),
        })
    }
}

impl GetPackageSectionMut for CargoToml {

    type Error = CargoTomlError;

    /// Helper to retrieve the `package` section from `Cargo.toml`
    fn get_package_section_mut(&mut self) -> Result<&mut toml::Value, Self::Error> {

        let possible_error = CargoTomlError::MissingPackageSection {
            cargo_toml_file: self.path().clone(),
        };

        self.content_mut().get_mut("package").ok_or_else(|| possible_error)
    }
}

impl GatherBinTargetNames for CargoToml {
    type Error = CargoTomlError;

    fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error> {
        let binding = Vec::new();
        // 1) Look up `[bin]` array if it exists
        let bin_array = self
            .content()
            .get("bin")
            .and_then(|val| val.as_array())
            .unwrap_or(&binding);

        // 2) For each bin entry, if there's a `name` field, we produce "name.rs"
        let mut result = Vec::new();
        for bin_val in bin_array {
            if let Some(tbl) = bin_val.as_table() {
                if let Some(name_val) = tbl.get("name").and_then(|nv| nv.as_str()) {
                    let fname = format!("{}.rs", name_val);
                    result.push(fname);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test_get_package_section {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;
    use tokio::runtime::Runtime;

    /// Helper function to create a Cargo.toml file with given contents,
    /// then build a `CargoToml` from it asynchronously.
    /// Returns the created `CargoToml` or an error.
    async fn create_cargo_toml_file(contents: &str) -> Result<CargoToml, CargoTomlError> {
        // Create a temporary file on disk
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp, "{}", contents).expect("Failed to write to temp file");
        let path = temp.into_temp_path();

        // We'll copy these bytes into a path we control in async style
        // so that `CargoToml::new` (which uses async fs ops) can read it.
        let path_buf = path.to_path_buf();
        let cargo_data = contents.as_bytes();

        // In an actual test scenario, you might just do `fs::write(path_buf, contents).await`,
        // but NamedTempFile requires a little extra dance to preserve the path.
        fs::write(&path_buf, cargo_data)
            .await
            .expect("Failed to write async to temp file");

        // Now create the CargoToml struct from this path
        CargoToml::new(&path_buf).await
    }

    /// Tests a valid `[package]` section is properly retrieved.
    #[tokio::test]
    async fn test_get_package_section_ok() {
        let toml_str = r#"
            [package]
            name = "test-crate"
            version = "0.1.0"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Failed to create CargoToml");

        // Now call get_package_section:
        let package_section = cargo_toml
            .get_package_section()
            .expect("Expected package section to be present");

        // Basic checks: we at least expect the `package` to have name & version
        assert!(package_section.is_table(), "package section should be a table");
        assert_eq!(
            package_section.get("name").and_then(|v| v.as_str()),
            Some("test-crate"),
            "package.name not matching"
        );
        assert_eq!(
            package_section.get("version").and_then(|v| v.as_str()),
            Some("0.1.0"),
            "package.version not matching"
        );
    }

    /// Tests that `get_package_section` returns an error if the `[package]` key is missing.
    #[tokio::test]
    async fn test_get_package_section_missing() {
        let toml_str = r#"
            [dependencies]
            serde = "1.0"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Failed to create CargoToml");

        let result = cargo_toml.get_package_section();
        match result {
            Ok(_) => panic!("Expected an error since there's no [package] section"),
            Err(e) => {
                // Ensure we got the right variant
                match e {
                    CargoTomlError::MissingPackageSection { .. } => {
                        // success: error variant is correct
                    }
                    _ => panic!("Unexpected error variant: {e:?}"),
                }
            }
        }
    }

    /// Tests that `get_package_section` still returns Ok if `[package]` is present,
    /// even if it’s not a table. (Implementation detail: it only checks existence,
    /// not that it’s actually a table.)
    #[tokio::test]
    async fn test_get_package_section_weird_but_present() {
        // This is contrived: "package" is just a string, not a table
        let toml_str = r#"
            package = "not-actually-a-table"
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Failed to create CargoToml");

        let package_section = cargo_toml
            .get_package_section()
            .expect("Should still find the 'package' key, even if not a table");

        // It's a string in this weird scenario
        assert_eq!(
            package_section.as_str(),
            Some("not-actually-a-table"),
            "Expected the 'package' entry to be a string"
        );
    }

    /// Tests that a minimal `[package]` table with no fields is still considered present.
    #[tokio::test]
    async fn test_get_package_section_empty_table() {
        let toml_str = r#"
            [package]
        "#;

        let cargo_toml = create_cargo_toml_file(toml_str).await
            .expect("Failed to create CargoToml");

        // Should be Ok, but it's an empty table
        let package_section = cargo_toml
            .get_package_section()
            .expect("Expected empty [package] to exist");

        assert!(package_section.is_table(), "Expected an empty table");
        let tbl = package_section.as_table().unwrap();
        assert!(tbl.is_empty(), "Should be an empty table");
    }
}
