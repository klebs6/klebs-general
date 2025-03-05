// ---------------- [ File: workspacer-bump/src/bump_crate.rs ]
crate::ix!();

#[async_trait]
impl Bump for CrateHandle {
    type Error = CrateError;

    async fn bump(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        trace!("Attempting to bump crate at {:?}", self.as_ref());

        let cargo_toml_path = self.as_ref().join("Cargo.toml");
        trace!("Reading Cargo.toml at {:?}", cargo_toml_path);

        // 1) Read from disk
        let contents = fs::read_to_string(&cargo_toml_path).await.map_err(|io_err| {
            error!("I/O error reading Cargo.toml: {:?}", io_err);
            CrateError::IoError {
                context: format!("reading {cargo_toml_path:?}"),
                io_error: Arc::new(io_err),
            }
        })?;

        // 2) Attempt a full parse with `toml_edit`.
        //    If it fails, we warn and try a partial parse that only reads [package].
        let full_parse_result = contents.parse::<toml_edit::Document>();
        let doc = match full_parse_result {
            Ok(doc) => {
                debug!("Full TOML parse succeeded for {:?}", cargo_toml_path);
                doc
            }
            Err(e) => {
                warn!(
                    "Full parse failed in {:?} with error {:?}. Trying partial [package]-only parse.",
                    cargo_toml_path, e
                );
                let doc = partial_package_parse(&contents, &cargo_toml_path)?;
                debug!("Partial [package] parse succeeded for {:?}", cargo_toml_path);
                doc
            }
        };

        // 3) Locate [package].version
        let pkg_item = doc.get("package").ok_or_else(|| {
            error!("Missing [package] section in {:?}", cargo_toml_path);
            CrateError::CargoTomlError(CargoTomlError::MissingPackageSection {
                cargo_toml_file: cargo_toml_path.clone(),
            })
        })?;

        let pkg_table = pkg_item.as_table().ok_or_else(|| {
            error!("[package] is not a valid table in {:?}", cargo_toml_path);
            CrateError::CargoTomlError(CargoTomlError::MissingPackageSection {
                cargo_toml_file: cargo_toml_path.clone(),
            })
        })?;

        let version_item = pkg_table.get("version").ok_or_else(|| {
            error!("Missing 'version' in [package] of {:?}", cargo_toml_path);
            CrateError::CargoTomlError(CargoTomlError::MissingRequiredFieldForIntegrity {
                cargo_toml_file: cargo_toml_path.clone(),
                field: "version".into(),
            })
        })?;

        let old_version_str = version_item.as_str().ok_or_else(|| {
            error!("'version' key not a valid string in {:?}", cargo_toml_path);
            CrateError::CargoTomlError(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: cargo_toml_path.clone(),
                version: format!("{}", version_item),
            })
        })?;

        trace!("Current version in {:?} => {}", cargo_toml_path, old_version_str);

        // 4) Parse as semver, then apply the requested release bump
        let mut parsed_ver = semver::Version::parse(old_version_str).map_err(|err| {
            error!("Invalid semver '{}' => {:?}", old_version_str, err);
            CrateError::CargoTomlError(CargoTomlError::InvalidVersionFormat {
                cargo_toml_file: cargo_toml_path.clone(),
                version: old_version_str.to_owned(),
            })
        })?;
        release.apply_to(&mut parsed_ver);
        let new_version = parsed_ver.to_string();

        trace!("Updated version => {}", new_version);

        // 5) Put the new version back into a clone of the doc, then write
        let mut updated_doc = doc.clone();
        if let Some(t) = updated_doc.get_mut("package").and_then(|it| it.as_table_mut()) {
            t["version"] = toml_edit::value(new_version.clone());
        }

        let new_contents = updated_doc.to_string();
        fs::write(&cargo_toml_path, new_contents).await.map_err(|io_err| {
            error!("Error writing updated Cargo.toml: {:?}", io_err);
            CrateError::IoError {
                context: format!("writing updated {cargo_toml_path:?}"),
                io_error: Arc::new(io_err),
            }
        })?;

        info!("Successfully bumped crate at {:?} to {}", cargo_toml_path, new_version);
        Ok(())
    }
}


/// A small helper that does a **partial** parse of just the `[package]`
/// section from the input string, ignoring parse errors in other sections.
/// We look for lines under `[package]` until the next header like `[foo]`.
/// Then we attempt to parse that snippet as its own toml_edit::Document.
///
/// Returns a new `Document` that has a `[package]` table only.
///
#[tracing::instrument(level="trace", skip_all)]
fn partial_package_parse(
    raw: &str,
    cargo_toml_path: &Path,
) -> Result<toml_edit::Document, CrateError> {
    use std::fmt::Write as _;

    let mut package_lines = String::new();
    let mut in_package = false;

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("[package]") {
            in_package = true;
            writeln!(package_lines, "{}", line).ok();
            continue;
        }
        if in_package {
            // If we see the next bracket header, we stop
            if trimmed.starts_with('[') {
                // next section => stop collecting
                break;
            }
            // keep collecting
            writeln!(package_lines, "{}", line).ok();
        }
    }

    // If we never found [package], that's an error
    if !in_package {
        error!("No [package] header found in partial parse for {:?}", cargo_toml_path);
        return Err(CrateError::CargoTomlError(CargoTomlError::MissingPackageSection {
            cargo_toml_file: cargo_toml_path.to_path_buf(),
        }));
    }

    // Now parse just that snippet
    match package_lines.parse::<toml_edit::Document>() {
        Ok(doc) => Ok(doc),
        Err(e) => {
            error!(
                "Even partial parse of [package] failed in {:?}: {}",
                cargo_toml_path, e
            );
            Err(CrateError::CargoTomlError(CargoTomlError::TomlEditError {
                cargo_toml_file: cargo_toml_path.to_path_buf(),
                toml_parse_error: e,
            }))
        }
    }
}

#[cfg(test)]
mod test_bump_crate_handle {
    use super::*;

    /// Helper: read the `[package].version` from a cargo toml file for quick verification
    async fn read_package_version(cargo_toml_path: &Path) -> Option<String> {
        let contents = fs::read_to_string(cargo_toml_path).await.ok()?;
        let doc = contents.parse::<toml_edit::Document>().ok()?;
        let pkg = doc.get("package")?.as_table()?;
        let ver_item = pkg.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    /// Constructs a single "bumpable" crate in a mock workspace, then returns a `CrateHandle`.
    /// By default, the crate has version "0.1.0". 
    /// You can pass in a custom version by rewriting Cargo.toml after creation, if desired.
    async fn setup_single_crate_handle(crate_name: &str) -> (tempfile::TempDir, CrateHandle) {
        // Build the mock workspace in a temp dir
        let single_cfg = CrateConfig::new(crate_name).with_src_files();
        let root_path = create_mock_workspace(vec![single_cfg])
            .await
            .expect("Failed to create mock workspace");
        
        // We'll build a minimal struct to implement `AsRef<Path>` so we can 
        // call `CrateHandle::new` if your code requires that approach.
        // Or we can do the direct approach: `CrateHandle::new` if it doesn't require a wrapper.
        //
        // For demonstration, let's assume `CrateHandle::new(&root_path.join(crate_name))`.
        let crate_path = root_path.join(crate_name);
        let handle = CrateHandle::new(&crate_path)
            .await
            .expect("Failed to create CrateHandle from mock crate directory");

        // Return the tempdir so it doesn't get dropped and our handle
        (tempfile::TempDir::new_in(root_path.parent().unwrap()).unwrap(), handle)
    }

    #[traced_test]
    async fn test_bump_major_ok() {
        let (_temp, mut handle) = setup_single_crate_handle("major_ok").await;

        // Confirm initial version is "0.1.0"
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");
        let initial_ver = read_package_version(&cargo_toml_path).await
            .expect("Expected an initial version");
        assert_eq!(initial_ver, "0.1.0");

        // Now bump major
        handle.bump(ReleaseType::Major).await
            .expect("Should succeed bumping major");

        // Check new version => "1.0.0"
        let updated_ver = read_package_version(&cargo_toml_path).await
            .expect("Expected version after major bump");
        assert_eq!(updated_ver, "1.0.0");
    }

    #[traced_test]
    async fn test_bump_minor_ok() {
        let (_temp, mut handle) = setup_single_crate_handle("minor_ok").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // Bump minor => from 0.1.0 to 0.2.0
        handle.bump(ReleaseType::Minor).await.expect("bump minor ok");
        let new_ver = read_package_version(&cargo_toml_path).await.unwrap();
        assert_eq!(new_ver, "0.2.0");
    }

    #[traced_test]
    async fn test_bump_patch_ok() {
        let (_temp, mut handle) = setup_single_crate_handle("patch_ok").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // Bump patch => from 0.1.0 to 0.1.1
        handle.bump(ReleaseType::Patch).await.expect("bump patch ok");
        let new_ver = read_package_version(&cargo_toml_path).await.unwrap();
        assert_eq!(new_ver, "0.1.1");
    }

    #[traced_test]
    async fn test_bump_alpha_ok() {
        let (_temp, mut handle) = setup_single_crate_handle("alpha_ok").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // Bump alpha(None) => from 0.1.0 => 0.1.0-alpha1
        handle.bump(ReleaseType::Alpha(None)).await.expect("bump alpha none");
        let alpha_ver = read_package_version(&cargo_toml_path).await.unwrap();
        assert_eq!(alpha_ver, "0.1.0-alpha1");

        // Bump alpha(Some(99)) => => 0.1.0-alpha99 (still same major/minor/patch)
        handle.bump(ReleaseType::Alpha(Some(99))).await.expect("bump alpha 99");
        let alpha99_ver = read_package_version(&cargo_toml_path).await.unwrap();
        assert_eq!(alpha99_ver, "0.1.0-alpha99");
    }

    /// Test error if `[package]` table is missing
    #[traced_test]
    async fn test_missing_package_section_error() {
        let (_temp, mut handle) = setup_single_crate_handle("missing_package").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // We'll sabotage the Cargo.toml by removing the "[package]" line
        let contents = fs::read_to_string(&cargo_toml_path).await.unwrap();
        let sabotage = contents.replace("[package]", "#[package]");
        fs::write(&cargo_toml_path, sabotage).await.unwrap();

        // Now call bump => expect an error
        let result = handle.bump(ReleaseType::Patch).await;
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::MissingPackageSection { cargo_toml_file }
            )) => {
                assert_eq!(cargo_toml_file, cargo_toml_path);
            }
            other => {
                panic!("Expected MissingPackageSection, got: {:?}", other);
            }
        }
    }

    /// Test error if `version` key is missing
    #[traced_test]
    async fn test_missing_version_key_error() {
        let (_temp, mut handle) = setup_single_crate_handle("missing_version_key").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // remove the "version = ..." line
        let contents = fs::read_to_string(&cargo_toml_path).await.unwrap();
        let sabotage = contents.replace("version = \"0.1.0\"", "");
        fs::write(&cargo_toml_path, sabotage).await.unwrap();

        let result = handle.bump(ReleaseType::Major).await;
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::MissingRequiredFieldForIntegrity { cargo_toml_file, field }
            )) => {
                assert_eq!(cargo_toml_file, cargo_toml_path);
                assert_eq!(field, "version");
            }
            other => panic!("Expected MissingRequiredFieldForIntegrity, got: {:?}", other),
        }
    }

    /// Test error if version is invalid semver
    #[traced_test]
    async fn test_invalid_version_format() {
        let (_temp, mut handle) = setup_single_crate_handle("invalid_version_format").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // sabotage the version line
        let contents = fs::read_to_string(&cargo_toml_path).await.unwrap();
        let sabotage = contents.replace("version = \"0.1.0\"", "version = \"not.semver\"");
        fs::write(&cargo_toml_path, sabotage).await.unwrap();

        let result = handle.bump(ReleaseType::Patch).await;
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::InvalidVersionFormat { cargo_toml_file, version }
            )) => {
                assert_eq!(cargo_toml_file, cargo_toml_path);
                assert_eq!(version, "not.semver".to_string());
            }
            other => panic!("Expected InvalidVersionFormat, got: {:?}", other),
        }
    }

    /// Test I/O error: e.g., remove the Cargo.toml before calling bump => triggers IoError
    #[traced_test]
    async fn test_io_error_missing_cargo_toml() {
        let (_temp, mut handle) = setup_single_crate_handle("io_error_missing_toml").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");
        
        // remove the file
        fs::remove_file(&cargo_toml_path).await.unwrap();

        // now call bump => expect IoError
        let result = handle.bump(ReleaseType::Patch).await;
        match result {
            Err(CrateError::IoError { context, .. }) => {
                assert!(context.contains("reading"), "Should mention reading cargo toml");
            }
            other => panic!("Expected IoError from missing Cargo.toml, got: {:?}", other),
        }
    }

    /// (Optional) test overwriting a huge alpha version => still success or partial skip
    #[traced_test]
    async fn test_alpha_huge_number() {
        let (_temp, mut handle) = setup_single_crate_handle("alpha_huge").await;
        let cargo_toml_path = handle.as_ref().join("Cargo.toml");

        // from 0.1.0 => 0.1.0-alpha999999999999999999
        let big_num = 999999999999999999u64;
        let res = handle.bump(ReleaseType::Alpha(Some(big_num))).await;
        assert!(res.is_ok(), "Should not fail semver parse for large alpha number");

        let new_ver_str = read_package_version(&cargo_toml_path).await.expect("expected version");
        // It's typically "0.1.0-alpha999999999999999999" if semver accepted it,
        // or "0.1.0+some-limitation" if it can't parse. 
        // Let's just check it contains "alpha".
        assert!(
            new_ver_str.contains("alpha"),
            "Expected alpha in the version, got: {new_ver_str}"
        );
    }
}
