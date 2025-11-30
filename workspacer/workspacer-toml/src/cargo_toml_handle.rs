// ---------------- [ File: workspacer-toml/src/cargo_toml_handle.rs ]
crate::ix!();

/// Handle to manipulate and verify the `Cargo.toml` file
#[derive(Serialize,Deserialize,Builder,MutGetters,Getters,Setters,Debug,Clone)]
#[builder(setter(into))]
#[getset(set="pub",get="pub",get_mut="pub")]
pub struct CargoToml {
    path:    PathBuf,
    content: toml::Value,  // Parsed TOML content
}

impl CargoTomlInterface for CargoToml {}

impl GetContent for CargoToml {
    fn get_content(&self) -> &toml::Value {
        self.content()
    }
}

#[async_trait]
impl SaveToDisk for CargoToml {
    type Error = CargoTomlError;

    async fn save_to_disk(&self) -> Result<(), Self::Error> {
        // 1) Convert self.content => string
        let rendered = toml::to_string_pretty(&self.content).map_err(|e| {
            // Instead of `TomlRenderError`, use our new variant
            CargoTomlError::TomlSerializeError {
                message: format!("Could not render updated TOML: {e}"),
            }
        })?;

        // 2) Write to disk
        tokio::fs::write(&self.path, rendered)
            .await
            .map_err(|io_err| CargoTomlError::IoWriteError {
                path: self.path.clone(),
                source: Arc::new(io_err),
            })?;

        Ok(())
    }
}

impl ReferencesDependencyInAnyTable for CargoToml {
    /// Returns `true` if this manifest references `dep_name` in any of the
    /// standard dependency tables (top-level or target-specific):
    /// - `dependencies`
    /// - `dev-dependencies`
    /// - `build-dependencies`
    ///
    /// This is a read-only check; it does **not** inject or modify `version`
    /// fields. It treats path/git/workspace-only entries as "referenced".
    fn references_dependency_in_any_table(
        &self,
        dep_name: &str,
    ) -> Result<bool, CargoTomlError> {
        // Root must be a table
        let root_table = self
            .content
            .as_table()
            .ok_or_else(|| CargoTomlError::TopLevelNotATable {
                path: self.path.clone(),
                details: "Top-level TOML is not a table".to_string(),
            })?;

        // Helper to check a single table for a dependency key
        let mut table_has_dep = |tbl: &toml::value::Table| -> bool {
            for section_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(section_val) = tbl.get(section_key) {
                    if let Some(dep_tbl) = section_val.as_table() {
                        if dep_tbl.contains_key(dep_name) {
                            return true;
                        }
                    }
                }
            }
            false
        };

        // 1) Top-level dependency sections
        if table_has_dep(root_table) {
            return Ok(true);
        }

        // 2) target.'...'.{dependencies,dev-dependencies,build-dependencies}
        if let Some(targets) = root_table.get("target").and_then(|v| v.as_table()) {
            for (_target_name, target_tbl_val) in targets.iter() {
                if let Some(target_tbl) = target_tbl_val.as_table() {
                    if table_has_dep(target_tbl) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

impl UpdateDependencyVersionRaw for CargoToml {
    type Error = CargoTomlError;

    fn update_dependency_version(
        &mut self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<bool, Self::Error> {

        // We’re using `serde::toml::Value` here, not `toml_edit`, so we can't do `as_inline_table_mut()`.
        // Instead, we do:
        let root_table = self
            .content
            .as_table_mut()
            .ok_or_else(|| CargoTomlError::TopLevelNotATable {
                path: self.path.clone(),
                details: "Top-level TOML is not a table".to_string(),
            })?;

        let mut changed = false;
        for section_key in &["dependencies", "dev-dependencies", "build-dependencies"] {
            // Get section as table
            if let Some(section_val) = root_table.get_mut(*section_key) {
                if let Some(dep_table) = section_val.as_table_mut() {
                    // Check if this crate is listed
                    if let Some(dep_item) = dep_table.get_mut(dep_name) {
                        // If it’s a plain string version: replace it
                        if let Some(s) = dep_item.as_str() {
                            let _ = s; // just to emphasize the branch
                            *dep_item = toml::Value::String(new_version.into());
                            changed = true;
                        }
                        // If it’s a table/inline-table, update **only if** `version` exists
                        else if let Some(tbl) = dep_item.as_table_mut() {
                            if let Some(v) = tbl.get_mut("version") {
                                *v = toml::Value::String(new_version.into());
                                changed = true;
                            } else {
                                // path-only/git-only/workspace-only: leave unchanged
                                tracing::trace!(
                                    section = *section_key,
                                    dep = dep_name,
                                    "leaving dependency without `version` unchanged"
                                );
                            }
                        }

                        // else: could also be something else, e.g. a bool or int?
                        // We can choose to skip or do something else
                    }
                }
            }
        }

        Ok(changed)
    }
}

#[async_trait]
impl WriteDocumentBack for CargoToml {

    type Error = CargoTomlError;

    async fn write_document_back(&mut self, doc: &toml_edit::Document) 
        -> Result<(),Self::Error> 
    {
        let doc_str = doc.to_string();
        debug!("Writing pinned TOML back to {:?}", self.as_ref());
        Ok(
            tokio::fs::write(self.as_ref(), doc_str)
            .await
            .map_err(|ioe| CargoTomlWriteError::WriteError {
                io: ioe.into(),
                cargo_toml_file: self.as_ref().to_path_buf(),
            })?
        )
    }
}

#[async_trait]
impl DocumentClone for CargoToml {

    type Error = CargoTomlError;

    async fn document_clone(&self) -> Result<toml_edit::Document,Self::Error> {

        let original = tokio::fs::read_to_string(self.as_ref())
            .await
            .map_err(|ioe| CargoTomlError::ReadError { path: self.as_ref().to_path_buf(), io: ioe.into() })?;

        let parse_result = original.parse::<toml_edit::Document>();

        Ok(parse_result
            .map_err(|parse_err| CargoTomlError::TomlEditError {
                cargo_toml_file: self.as_ref().to_path_buf(),
                toml_parse_error: parse_err,
            })?)
    }
}

impl Versioned for CargoToml {
    type Error = CargoTomlError;

    fn version(&self) -> Result<semver::Version, Self::Error> {
        trace!("CargoToml::version: forcing a fresh read from disk");

        // 1) Check if the file even exists on disk. If not, return error:
        if !std::fs::metadata(&self.path).is_ok() {
            error!(
                "CargoToml::version => file not found at path={:?}",
                self.path
            );
            return Err(CargoTomlError::FileNotFound {
                missing_file: self.path.clone(),
            });
        }

        // 2) Read the entire file from disk:
        let contents = std::fs::read_to_string(&self.path).map_err(|io_err| {
            error!("CargoToml::version => read_to_string failed: {}", io_err);
            CargoTomlError::ReadError {
                path: self.path.clone(),
                io: Arc::new(io_err),
            }
        })?;

        // 3) Parse into toml_edit::Document to find [package].version:
        let doc = contents.parse::<toml_edit::Document>().map_err(|parse_e| {
            error!(
                "CargoToml::version => TOML parse error: {}",
                parse_e
            );
            CargoTomlError::InvalidToml {
                path: self.path.clone(),
                details: parse_e.to_string(),
            }
        })?;

        // 4) Grab the version string:
        let version_str = doc
            .get("package")
            .and_then(|val| val.as_table())
            .and_then(|tbl| tbl.get("version"))
            .and_then(|ver| ver.as_str())
            .ok_or_else(|| {
                error!("CargoToml::version => no 'version' key in [package] table");
                CargoTomlError::MissingRequiredFieldForIntegrity {
                    cargo_toml_file: self.path.clone(),
                    field: "version".to_string(),
                }
            })?;

        debug!("CargoToml::version - read version_str='{}' from disk for {:?}", version_str, self.path);

        // 5) Parse semver:
        match semver::Version::parse(version_str) {
            Ok(ver) => {
                info!("CargoToml::version => parsed version={} for {:?}", ver, self.path);
                Ok(ver)
            }
            Err(_e) => {
                error!("CargoToml::version => invalid semver: '{}'", version_str);
                Err(CargoTomlError::InvalidVersionFormat {
                    cargo_toml_file: self.path.clone(),
                    version: version_str.into(),
                })
            }
        }
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
            .map_err(|e| CargoTomlError::ReadError { path: cargo_toml_path.as_ref().to_path_buf(), io: e.into() })?;

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

    pub fn new_sync<P>(cargo_toml_path: P) -> Result<Self, CargoTomlError> 
        where P: AsRef<Path>
    {
        let cargo_content = std::fs::read_to_string(&cargo_toml_path)
            .map_err(|e| CargoTomlError::ReadError { 
                path: cargo_toml_path.as_ref().to_path_buf(), 
                io: e.into() 
            })?;

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

    #[tokio::test]
    async fn update_dep_does_not_inject_version_for_path_only_in_all_sections() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let toml_content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            a = { path = "../a" }
            b = { path = "../b", version = "1.0.0" }
            c = "0.2.3"

            [dev-dependencies]
            a = { path = "../a" }

            [build-dependencies]
            a = { path = "../a" }
        "#;

        let path = dir.path().join("Cargo.toml");
        tokio::fs::write(&path, toml_content).await.unwrap();
        let mut ct = CargoToml::new(&path).await.unwrap();

        // Update all three names to a new version
        let _ = ct.update_dependency_version("a", "9.9.9").unwrap();
        let _ = ct.update_dependency_version("b", "9.9.9").unwrap();
        let _ = ct.update_dependency_version("c", "9.9.9").unwrap();

        // No need to save to disk; we can inspect ct.content in memory
        let root = ct.get_content().as_table().unwrap();

        // deps.a remains path-only: no 'version'
        let deps = root.get("dependencies").unwrap().as_table().unwrap();
        assert!(deps.get("a").unwrap().as_table().unwrap().get("version").is_none());
        // deps.b had version: updated
        assert_eq!(
            deps.get("b").unwrap().as_table().unwrap().get("version").unwrap().as_str(),
            Some("9.9.9")
        );
        // deps.c was string: updated
        assert_eq!(deps.get("c").unwrap().as_str(), Some("9.9.9"));

        // dev-dependencies: a remains path-only
        let dev = root.get("dev-dependencies").unwrap().as_table().unwrap();
        assert!(dev.get("a").unwrap().as_table().unwrap().get("version").is_none());

        // build-dependencies: a remains path-only
        let build = root.get("build-dependencies").unwrap().as_table().unwrap();
        assert!(build.get("a").unwrap().as_table().unwrap().get("version").is_none());
    }

    #[tokio::test]
    async fn update_dep_does_not_inject_version_for_git_only() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let toml_content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            g = { git = "https://example.com/repo.git", rev = "abc123" }
        "#;
        let path = dir.path().join("Cargo.toml");
        tokio::fs::write(&path, toml_content).await.unwrap();
        let mut ct = CargoToml::new(&path).await.unwrap();

        let _ = ct.update_dependency_version("g", "1.2.3").unwrap();

        let deps = ct.get_content()
            .as_table().unwrap()
            .get("dependencies").unwrap()
            .as_table().unwrap();

        // Ensure no 'version' got injected
        assert!(deps.get("g").unwrap().as_table().unwrap().get("version").is_none());
    }
}
