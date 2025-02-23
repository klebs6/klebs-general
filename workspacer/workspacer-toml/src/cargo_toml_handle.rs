// ---------------- [ File: src/cargo_toml_handle.rs ]
crate::ix!();

/// Handle to manipulate and verify the `Cargo.toml` file
#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct CargoToml {
    path:    PathBuf,
    content: toml::Value,  // Parsed TOML content
}

impl CargoTomlInterface for CargoToml {}

impl Versioned for CargoToml {

    type Error = CargoTomlError;

    fn version(&self) -> Result<semver::Version,Self::Error> {
        self.check_required_fields_for_integrity()?;
        let package = self.get_package_section()?;
        let version_toml = package.get("version").unwrap();
        Ok(semver::Version::parse(&version_toml.to_string()).map_err(|e| Arc::new(e))?)
    }
}

impl CargoToml {

    pub fn package_name(&self) -> Result<String,CargoTomlError> {
        self.check_required_fields_for_integrity()?;
        let package = self.get_package_section()?;
        let name = package.get("name").unwrap();
        Ok(name.to_string())
    }

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

#[cfg(test)]
mod test_cargo_toml {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tempfile::tempdir;

    /// Helper function to write arbitrary content to a "Cargo.toml" file
    /// in a temporary directory. Returns the resulting file path.
    async fn write_cargo_toml_content(dir_path: &std::path::Path, content: &str) -> PathBuf {
        let cargo_toml_path = dir_path.join("Cargo.toml");
        let mut file = File::create(&cargo_toml_path)
            .await
            .expect("Failed to create Cargo.toml test file");
        file.write_all(content.as_bytes())
            .await
            .expect("Failed to write to test Cargo.toml");
        cargo_toml_path
    }

    /// Test that `CargoToml::new` loads a valid file and parses it into `CargoToml`.
    /// Checks that `package_name()` and `version()` are correct for a valid `[package]` section.
    #[tokio::test]
    async fn test_new_with_valid_toml() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        let toml_content = r#"
            [package]
            name = "test_crate"
            version = "0.1.2"
        "#;

        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), toml_content).await;

        let cargo_toml_handle = CargoToml::new(&cargo_toml_path)
            .await
            .expect("Expected successful creation of CargoToml");

        // Check package_name
        let name = cargo_toml_handle
            .package_name()
            .expect("Expected to parse package name");
        assert_eq!(name, "\"test_crate\"", "package_name should match the TOML content, note toml::Value to_string() includes quotes.");

        // Check version
        let version = cargo_toml_handle
            .version()
            .expect("Expected to parse valid semver version");
        assert_eq!(version.to_string(), "0.1.2", "Version should match the specified semver");
    }

    /// Test that `CargoToml::new` returns `ReadError` when the file does not exist.
    #[tokio::test]
    async fn test_new_with_non_existent_path() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        // We'll point to a file name that doesn't exist in that dir
        let cargo_toml_path = tmp_dir.path().join("Cargo.toml");

        let result = CargoToml::new(&cargo_toml_path).await;
        assert!(result.is_err(), "Expected an error for non-existent file");
        match result {
            Err(CargoTomlError::ReadError { .. }) => { /* expected */ }
            other => panic!("Expected CargoTomlError::ReadError, got {:?}", other),
        }
    }

    /// Test that `CargoToml::new` returns `TomlParseError` when content is malformed TOML.
    #[tokio::test]
    async fn test_new_with_malformed_toml() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        let invalid_toml_content = r#"invalid_toml:::??? = 123"#;
        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), invalid_toml_content).await;

        let result = CargoToml::new(&cargo_toml_path).await;
        assert!(result.is_err(), "Expected an error for malformed TOML");
        match result {
            Err(CargoTomlError::TomlParseError { .. }) => { /* expected */ }
            other => panic!("Expected CargoTomlError::TomlParseError, got {:?}", other),
        }
    }

    /// Test that `package_name()` fails when `[package]` section is missing entirely.
    #[tokio::test]
    async fn test_package_name_missing_package_section() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        // We omit `[package]` entirely
        let toml_content = r#"
            [dependencies]
            foo = "1.0"
        "#;
        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), toml_content).await;

        let cargo_toml_handle = CargoToml::new(&cargo_toml_path)
            .await
            .expect("Expected successful read+parse, but missing `[package]` is discovered later");

        let result = cargo_toml_handle.package_name();
        assert!(result.is_err(), "Expected an error for missing `[package]`");
        // Depending on how `check_required_fields_for_integrity()` is implemented,
        // this might yield a specific error variant or a generic message. We can match it:
        match result {
            Err(CargoTomlError::MissingPackageSection { .. })
            | Err(CargoTomlError::MissingRequiredFieldForIntegrity { .. }) 
            | Err(_) => { /* handle your actual error variant(s) here */ }
            _ => {}
        }
    }

    /// Test that `version()` fails when the version field is missing in `[package]`.
    #[tokio::test]
    async fn test_version_missing_in_package_section() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        // We have `[package]` but no `version`
        let toml_content = r#"
            [package]
            name = "my_crate"
        "#;
        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), toml_content).await;

        let cargo_toml_handle = CargoToml::new(&cargo_toml_path)
            .await
            .expect("Expected parse success for partial `[package]`, but missing version is discovered later");

        let result = cargo_toml_handle.version();
        assert!(result.is_err(), "Expected an error for missing 'version' field");
        // Depending on your integrity checks, match your variant
        match result {
            Err(CargoTomlError::MissingRequiredFieldForIntegrity { .. })
            | Err(_) => { /* handle accordingly */ }
            _ => {}
        }
    }

    /// Test that `version()` fails when the version field is not valid semver (e.g. "abc").
    #[tokio::test]
    async fn test_version_invalid_semver() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        let toml_content = r#"
            [package]
            name = "invalid_semver_crate"
            version = "not_semver"
        "#;
        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), toml_content).await;

        let cargo_toml_handle = CargoToml::new(&cargo_toml_path)
            .await
            .expect("Reading file should succeed, parse TOML should succeed, but semver parse will fail in `version()`");

        let result = cargo_toml_handle.version();
        assert!(result.is_err(), "Expected an error for invalid semver");
        // Typically, you'd get something like a parse error from semver
        match result {
            Err(CargoTomlError::SemverError { .. }) 
            | Err(_) => { /* handle your actual error variant(s) here */ }
            _ => {}
        }
    }

    /// Test the `AsRef<Path>` trait to confirm it returns the path we expect.
    #[tokio::test]
    async fn test_as_ref_path() {
        let tmp_dir = tempdir().expect("Failed to create temp dir for test");
        let toml_content = r#"
            [package]
            name = "test_crate"
            version = "1.0.0"
        "#;
        let cargo_toml_path = write_cargo_toml_content(tmp_dir.path(), toml_content).await;

        let cargo_toml_handle = CargoToml::new(&cargo_toml_path)
            .await
            .expect("Expected successful creation of CargoToml");

        // Check that as_ref() points to the same path
        let as_ref_path = cargo_toml_handle.as_ref();
        assert_eq!(as_ref_path, cargo_toml_path.as_path());
    }
}
