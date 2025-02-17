// ---------------- [ File: workspacer-toml/src/pin_wildcard_deps.rs ]
crate::ix!();

use std::collections::{BTreeMap, BTreeSet};
use toml_edit::{Document, Item, Table, Value};
use semver::Version;
use cargo_lock::{Lockfile, Package};

#[async_trait]
impl PinWildcardDependencies for CargoToml {

    type Error = CargoTomlError;

    /// Pin all wildcard dependencies ("*") in this crate's Cargo.toml
    /// to the highest version found in `lock_versions`.
    ///
    /// - If multiple distinct versions exist for a crate, logs a warning and uses the highest.
    /// - If a wildcard crate isn't found in the lock map, logs a warning.
    /// - Preserves formatting/comments as best as `toml_edit` allows.
    ///
    /// NOTE: This writes the updated TOML **in-place** to `self.path`.
    async fn pin_wildcard_dependencies(
        &self,
        lock_versions: &BTreeMap<String, BTreeSet<Version>>,
    ) -> Result<(), Self::Error> {
        // 1) Read original TOML text so we can re-parse with toml_edit
        let original_toml_str = tokio::fs::read_to_string(self.as_ref()).await
            .map_err(|e| CargoTomlError::ReadError { io: e.into() })?;

        // 2) Parse as a toml_edit::Document to preserve formatting
        let mut doc = original_toml_str
            .parse::<Document>()
            .map_err(|parse_err| CargoTomlError::TomlEditError {
                cargo_toml_file: self.as_ref().to_path_buf(),
                toml_parse_error: parse_err,
            })?;

        // 3) Pin the wildcards in the Document
        pin_wildcards_in_doc(&mut doc, lock_versions);

        // 4) Write updated TOML back in original order
        let pinned_str = doc.to_string(); 

        tokio::fs::write(self.as_ref(), pinned_str).await
            .map_err(|e| CargoTomlWriteError::WriteError {
                io: e.into(),
                cargo_toml_file: self.as_ref().to_path_buf(),
            })?;

        Ok(())
    }
}

/// This helper updates all wildcard dependencies in a `Document`.
fn pin_wildcards_in_doc(
    doc:           &mut Document,
    lock_versions: &BTreeMap<String, BTreeSet<Version>>,
) {
    // Fix top-level sections like [dependencies], [dev-dependencies], etc.
    for (key, item) in doc.as_table_mut().iter_mut() {
        if is_dependencies_key(&key) {
            if let Item::Table(dep_table) = item {
                pin_deps_in_table(dep_table, lock_versions);
            }
        }
    }

    // Also fix any nested dependencies in [target.'cfg(...)'.dependencies], etc.
    fix_nested_tables(doc.as_item_mut(), lock_versions);
}

/// Checks if a table key ends with "dependencies"
fn is_dependencies_key(k: &str) -> bool {
    k.ends_with("dependencies")
}

/// Recursively walks a `toml_edit::Item` looking for tables named "*dependencies".
fn fix_nested_tables(item: &mut Item, lock_versions: &BTreeMap<String, BTreeSet<Version>>) {
    if let Item::Table(tbl) = item {
        for (sub_key, sub_item) in tbl.iter_mut() {
            if is_dependencies_key(&sub_key) {
                if let Item::Table(dep_table) = sub_item {
                    pin_deps_in_table(dep_table, lock_versions);
                }
            }
            // Recurse deeper
            fix_nested_tables(sub_item, lock_versions);
        }
    }
}

/// Walk each dependency in a table, pinning wildcard versions.
fn pin_deps_in_table(
    table: &mut Table,
    lock_versions: &BTreeMap<String, BTreeSet<Version>>,
) {
    for (dep_name, dep_item) in table.iter_mut() {
        match dep_item {
            // e.g. `dep = "*"`
            Item::Value(Value::String { .. }) => {
                let current_str = dep_item.as_str().unwrap_or("");
                if current_str == "*" {
                    if let Some(new_ver) = pick_highest_version(&dep_name, lock_versions) {
                        *dep_item = Item::Value(Value::from(new_ver));
                    } else {
                        warn!(
                            "wildcard dep '{}' was not found in Cargo.lock",
                            dep_name
                        );
                    }
                }
            }
            // e.g. `dep = { version = "*", features = ["foo"] }`
            Item::Value(Value::InlineTable(inline_tab)) => {
                if let Some(version_item) = inline_tab.get("version") {
                    if version_item.as_str() == Some("*") {
                        if let Some(new_ver) = pick_highest_version(&dep_name, lock_versions) {
                            inline_tab.insert("version", Value::from(new_ver));
                        } else {
                            warn!(
                                "wildcard dep '{}' was not found in Cargo.lock",
                                dep_name
                            );
                        }
                    }
                }
            }
            _ => {
                // Possibly dotted tables or other forms. Usually do nothing.
            }
        }
    }
}

/// Choose the highest SemVer version from lock_versions for `crate_name`.
/// If multiple distinct versions are found, logs a warning and picks the highest.
fn pick_highest_version(
    crate_name: &str,
    lock_map: &BTreeMap<String, BTreeSet<Version>>,
) -> Option<String> {
    let set = lock_map.get(crate_name)?;
    if set.len() > 1 {
        warn!(
            "crate '{}' has multiple versions in Cargo.lock: {:?}. Using the highest.",
            crate_name, set
        );
    }
    let highest = set.iter().max().unwrap(); // safe if nonempty
    Some(highest.to_string())
}

#[cfg(test)]
mod test_pin_wildcard_dependencies {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::fs;

    /// Creates a real temp file with `contents` and returns (the file handle, the CargoToml).
    ///
    /// We store the `NamedTempFile` so that the file remains alive for the entire test,
    /// avoiding "No such file or directory" issues.
    async fn create_cargo_toml_file(contents: &str)
        -> Result<(NamedTempFile, CargoToml), CargoTomlError>
    {
        // 1) Create a NamedTempFile
        let mut temp_file = NamedTempFile::new().expect("failed to create NamedTempFile");

        // 2) Write synchronously
        write!(temp_file, "{}", contents).expect("failed to write to temp file");
        temp_file.flush().expect("failed to flush temp file");

        // 3) Build path from temp_file
        let path_buf = temp_file.path().to_path_buf();

        // 4) Let CargoToml read it
        let cargo_toml = CargoToml::new(&path_buf).await?;

        // 5) Return both so the file isn't dropped
        Ok((temp_file, cargo_toml))
    }

    /// Helper to build a mock lock-versions map: crate_name -> set_of_versions
    fn mock_lock_versions(data: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<Version>> {
        let mut map = BTreeMap::new();
        for (crate_name, vers) in data {
            let mut set = BTreeSet::new();
            for v in *vers {
                set.insert(Version::parse(v).unwrap());
            }
            map.insert(crate_name.to_string(), set);
        }
        map
    }

    /// 1) Test parse error
    #[tokio::test]
    async fn test_toml_parse_error() {
        let invalid_toml = r#"
            [package   # missing closing bracket
            name = "broken"
        "#;

        // We must confirm the parse step fails with TomlEditError:
        match create_cargo_toml_file(invalid_toml).await {
            Ok((_file, _toml)) => {
                panic!("Expected parse error, got Ok");
            }
            Err(e) => match e {
                CargoTomlError::TomlEditError  { .. } => { /* success */ }
                CargoTomlError::TomlParseError { .. } => { /* success */ }
                _ => panic!("Unexpected error variant: {e:?}"),
            }
        }
    }

    /// 2) Test a file with no wildcard dependencies -> pinning is a no-op, but success.
    #[tokio::test]
    async fn test_no_wildcards() {
        let contents = r#"
            [package]
            name = "no-wildcards"
            version = "1.0.0"

            [dependencies]
            serde = "1.0"
        "#;

        let (file, cargo_toml) = create_cargo_toml_file(contents).await
            .expect("Should parse valid TOML");
        let lock_versions = mock_lock_versions(&[
            ("serde", &["1.0.188"]),
        ]);

        // Pin -> should do nothing, but succeed
        cargo_toml
            .pin_wildcard_dependencies(&lock_versions)
            .await
            .expect("Should succeed with no wildcards");

        // Re-read file to confirm no changes
        let pinned = fs::read_to_string(file.path())
            .await
            .expect("Failed to read pinned file");
        assert_eq!(pinned, contents);
    }

    /// 3) Single wildcard pinned from lock
    #[tokio::test]
    async fn test_single_wildcard() {
        let contents = r#"
            [package]
            name = "single-wildcard"
            version = "0.1.0"

            [dependencies]
            foo = "*"
        "#;

        let (file, cargo_toml) = create_cargo_toml_file(contents).await
            .expect("valid TOML");
        let lock_versions = mock_lock_versions(&[
            ("foo", &["0.5.0"]),
        ]);

        cargo_toml
            .pin_wildcard_dependencies(&lock_versions)
            .await
            .expect("pinning should succeed for single wildcard");

        let pinned = fs::read_to_string(file.path())
            .await
            .expect("Re-read pinned file");
        // We should see foo pinned to "0.5.0"
        assert!(pinned.contains(r#"foo = "0.5.0""#));
        assert!(!pinned.contains(r#"foo = "*""#));
    }

    /// 4) Multiple distinct versions in lock -> picks highest
    #[tokio::test]
    async fn test_multiple_lock_versions() {
        let contents = r#"
            [package]
            name = "multi-versions"
            version = "0.1.0"

            [dependencies]
            bar = "*"
        "#;

        let (file, cargo_toml) = create_cargo_toml_file(contents).await
            .expect("valid TOML");
        let lock_versions = mock_lock_versions(&[
            ("bar", &["0.9.0", "1.2.3", "1.5.2"]),
        ]);

        cargo_toml
            .pin_wildcard_dependencies(&lock_versions)
            .await
            .expect("pinning should succeed picking highest (1.5.2)");

        let pinned = fs::read_to_string(file.path())
            .await
            .expect("read pinned file");
        assert!(pinned.contains(r#"bar = "1.5.2""#));
    }

    /// 5) Missing crate in lock -> remains "*"
    #[tokio::test]
    async fn test_missing_in_lock() {
        let contents = r#"
            [package]
            name = "missing-lock"
            version = "0.1.0"

            [dependencies]
            not_in_lock = "*"
        "#;

        let (file, cargo_toml) = create_cargo_toml_file(contents).await
            .expect("valid TOML");
        // no `not_in_lock` in the map
        let lock_versions = mock_lock_versions(&[
            ("some_other_crate", &["1.0.0"]),
        ]);

        cargo_toml
            .pin_wildcard_dependencies(&lock_versions)
            .await
            .expect("pin should not fail, just skip unknown crate");

        let pinned = fs::read_to_string(file.path())
            .await
            .expect("read pinned file");
        assert!(pinned.contains(r#"not_in_lock = "*""#));
    }

    /// 6) Nested wildcard in target
    #[tokio::test]
    async fn test_nested_wildcard() {
        let contents = r#"
            [package]
            name = "nested-wild"
            version = "0.1.0"

            [target."cfg(windows)".dependencies]
            windows_dep = "*"

            [target."cfg(unix)".dev-dependencies]
            nix_dep = "*"
        "#;

        let (file, cargo_toml) = create_cargo_toml_file(contents).await
            .expect("valid TOML");

        let lock_versions = mock_lock_versions(&[
            ("windows_dep", &["0.4.0"]),
            ("nix_dep", &["0.2.0"]),
        ]);

        cargo_toml
            .pin_wildcard_dependencies(&lock_versions)
            .await
            .expect("nested wildcard pinning should succeed");

        let pinned = fs::read_to_string(file.path())
            .await
            .expect("read pinned file");
        assert!(pinned.contains(r#"windows_dep = "0.4.0""#));
        assert!(pinned.contains(r#"nix_dep = "0.2.0""#));
    }
}
