crate::ix!();

#[async_trait]
pub trait GetVersionOfLocalDep {
    async fn version_of_local_dep(&self, dep_name: &str, dep_path: &str) -> Option<String>;
}

#[async_trait]
impl GetVersionOfLocalDep for CargoToml {
    /// Helper to create a `CargoToml` for a neighbor crate at `dep_path` (relative to our own Cargo.toml),
    /// then fetch that crate's version. If any errors occur, returns None and logs a warning.
    async fn version_of_local_dep(&self, dep_name: &str, dep_path: &str) -> Option<String> {
        let base_dir = self.path().parent().unwrap_or_else(|| self.path());
        let full_dep_path = base_dir.join(dep_path);

        debug!(
            "version_of_local_dep: Resolving path dependency for '{}' at {:?}",
            dep_name, full_dep_path
        );
        match CargoToml::new(&full_dep_path).await {
            Ok(dep_cargo_toml) => {
                match dep_cargo_toml.version() {
                    Ok(ver) => {
                        let vs = ver.to_string();
                        info!(
                            "version_of_local_dep: Found neighbor crate '{}' version='{}' at path {:?}",
                            dep_name, vs, full_dep_path
                        );
                        Some(vs)
                    }
                    Err(e) => {
                        warn!(
                            "version_of_local_dep: Failed to parse version for '{}': {:?}",
                            dep_name, e
                        );
                        None
                    }
                }
            }
            Err(e) => {
                warn!(
                    "version_of_local_dep: Could not open neighbor crate for '{}': {:?}",
                    dep_name, e
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod test_version_of_local_dep {
    use super::*;

    /// A minimal helper to write a Cargo.toml with certain content to the specified file path.
    async fn write_cargo_toml(file_path: &std::path::Path, contents: &str) {
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent).await.expect("Failed to create parent dirs");
        }
        let mut file = File::create(&file_path)
            .await
            .expect("Could not create Cargo.toml");
        file.write_all(contents.as_bytes())
            .await
            .expect("Failed to write Cargo.toml content");
    }

    /// Creates a minimal valid Cargo.toml with the given version in the given directory,
    /// returning the full path to that Cargo.toml.
    async fn create_local_crate(dir: &std::path::Path, version_str: &str) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = format!(
            r#"[package]
name = "local_dep"
version = "{}"
"#,
            version_str
        );
        write_cargo_toml(&cargo_toml_path, &content).await;
        cargo_toml_path
    }

    /// Same approach, but we intentionally omit the "version" field to test errors.
    async fn create_local_crate_no_version(dir: &std::path::Path) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = r#"
[package]
name = "local_dep_no_ver"
# no version
"#;
        write_cargo_toml(&cargo_toml_path, &content).await;
        cargo_toml_path
    }

    /// Creates a crate with an invalid semver version string
    async fn create_local_crate_invalid_semver(dir: &std::path::Path) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = r#"
[package]
name = "local_dep_invalid_semver"
version = "not_a_valid_semver"
"#;
        write_cargo_toml(&cargo_toml_path, &content).await;
        cargo_toml_path
    }

    /// Creates a crate with no [package] section at all
    async fn create_local_crate_no_package_section(dir: &std::path::Path) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = r#"
# This TOML has no [package] table
[dependencies]
foo = "1.2.3"
"#;
        write_cargo_toml(&cargo_toml_path, &content).await;
        cargo_toml_path
    }

    /// Creates a "root" CargoToml that will call `version_of_local_dep`.
    /// The path is artificially assigned to some location we control via a temp directory.
    async fn create_root_cargo_toml(dir: &std::path::Path) -> CargoToml {
        let root_cargo_toml_path = dir.join("RootCargo.toml");
        let content = r#"
[package]
name = "root_crate"
version = "0.1.0"
"#;
        write_cargo_toml(&root_cargo_toml_path, content).await;

        // Then build a CargoToml handle for that root cargo
        CargoToml::new(&root_cargo_toml_path)
            .await
            .expect("Failed to create root CargoToml handle")
    }

    // -------------------------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------------------------

    #[traced_test]
    async fn test_version_of_local_dep_valid_crate_happy_path() {
        info!("test_version_of_local_dep_valid_crate_happy_path: start");

        let tempdir_root = tempdir().expect("Failed to create temp dir for root crate");
        let tempdir_local = tempdir().expect("Failed to create temp dir for local dep crate");

        // 1) Create a local crate with a known version
        let local_cargo_toml_path = create_local_crate(tempdir_local.path(), "1.2.3").await;

        // 2) Create a "root" cargo toml
        let root_cargo_toml = create_root_cargo_toml(tempdir_root.path()).await;

        // 3) relative dep path to pass in
        //    We'll set it to be something from the root's directory to the local's directory
        let rel_path = pathdiff::diff_paths(&local_cargo_toml_path.parent().unwrap(), root_cargo_toml.path().parent().unwrap())
            .expect("Failed to compute relative path");

        let version_opt = root_cargo_toml.version_of_local_dep("local_dep", rel_path.to_str().unwrap()).await;
        assert!(version_opt.is_some(), "Expected to get Some(version) for a valid local crate");
        let ver_str = version_opt.unwrap();
        assert_eq!(ver_str, "1.2.3");

        info!("test_version_of_local_dep_valid_crate_happy_path: done");
    }

    /// If the local crate's Cargo.toml is missing or can't be opened,
    /// we expect version_of_local_dep to return None and log a warning.
    #[traced_test]
    async fn test_version_of_local_dep_failed_to_open() {
        info!("test_version_of_local_dep_failed_to_open: start");

        let tempdir_root = tempdir().expect("Failed to create temp dir for root crate");
        let root_cargo_toml = create_root_cargo_toml(tempdir_root.path()).await;

        // We'll pass a path that doesn't exist
        let non_existent_path = "this_path_is_not_real/does_not_exist";

        let version_opt = root_cargo_toml
            .version_of_local_dep("dep_missing_path", non_existent_path)
            .await;
        assert!(version_opt.is_none(), "Should return None if we can't open neighbor crate Cargo.toml");

        info!("test_version_of_local_dep_failed_to_open: done");
    }

    /// If the local crate has an invalid semver in its [package] version,
    /// CargoToml::version() will fail, causing version_of_local_dep to log a warning and return None.
    #[traced_test]
    async fn test_version_of_local_dep_invalid_semver() {
        info!("test_version_of_local_dep_invalid_semver: start");

        let tempdir_root = tempdir().expect("Failed to create temp dir for root crate");
        let tempdir_local = tempdir().expect("Failed to create temp dir for local dep crate");

        let local_cargo_toml_path = create_local_crate_invalid_semver(tempdir_local.path()).await;
        let root_cargo_toml = create_root_cargo_toml(tempdir_root.path()).await;

        let rel_path = pathdiff::diff_paths(&local_cargo_toml_path.parent().unwrap(), root_cargo_toml.path().parent().unwrap())
            .expect("diff path fail");
        let version_opt = root_cargo_toml
            .version_of_local_dep("invalid_semver_dep", rel_path.to_str().unwrap())
            .await;

        assert!(version_opt.is_none(), "Expected None for invalid semver local crate");
        info!("test_version_of_local_dep_invalid_semver: done");
    }

    /// If the local crate has no "version" in the [package] section,
    /// then `CargoToml::version()` typically fails. We expect a warning and None.
    #[traced_test]
    async fn test_version_of_local_dep_no_version_in_package() {
        info!("test_version_of_local_dep_no_version_in_package: start");

        let tempdir_root = tempdir().expect("Failed to create temp dir for root crate");
        let tempdir_local = tempdir().expect("Failed to create temp dir for local dep crate");

        let local_cargo_toml_path = create_local_crate_no_version(tempdir_local.path()).await;
        let root_cargo_toml = create_root_cargo_toml(tempdir_root.path()).await;

        let rel_path = pathdiff::diff_paths(&local_cargo_toml_path.parent().unwrap(), root_cargo_toml.path().parent().unwrap())
            .unwrap();
        let version_opt = root_cargo_toml
            .version_of_local_dep("dep_no_version", rel_path.to_str().unwrap())
            .await;

        assert!(version_opt.is_none(), "Should return None if local crate has no version field");
        info!("test_version_of_local_dep_no_version_in_package: done");
    }

    /// If the local crate has no [package] section at all,
    /// then reading the version will fail, causing None to be returned.
    #[traced_test]
    async fn test_version_of_local_dep_missing_package_section() {
        info!("test_version_of_local_dep_missing_package_section: start");

        let tempdir_root = tempdir().expect("Failed to create temp dir for root crate");
        let tempdir_local = tempdir().expect("Failed to create temp dir for local dep crate");

        let local_cargo_toml_path = create_local_crate_no_package_section(tempdir_local.path()).await;
        let root_cargo_toml = create_root_cargo_toml(tempdir_root.path()).await;

        let rel_path = pathdiff::diff_paths(&local_cargo_toml_path.parent().unwrap(), root_cargo_toml.path().parent().unwrap())
            .expect("Failed to compute relative path");
        let version_opt = root_cargo_toml
            .version_of_local_dep("dep_no_package_section", rel_path.to_str().unwrap())
            .await;
        assert!(version_opt.is_none(), "Should return None if there's no [package] table at all");
        info!("test_version_of_local_dep_missing_package_section: done");
    }
}
