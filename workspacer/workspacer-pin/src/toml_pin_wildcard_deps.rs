// ---------------- [ File: workspacer-pin/src/toml_pin_wildcard_deps.rs ]
crate::ix!();

use std::collections::{BTreeMap, BTreeSet};
use toml_edit::{Document, Item, Table, Value};
use semver::Version;
use cargo_lock::{Lockfile, Package};

pub type LockVersionMap = BTreeMap<String, BTreeSet<semver::Version>>;

#[async_trait]
pub trait PinWildcardDependencies {

    type Error;

    async fn pin_wildcard_dependencies(
        &self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), Self::Error>;
}


#[async_trait]
impl PinWildcardDependencies for CargoToml {
    type Error = CargoTomlError;

    /**
     * Pins all wildcard dependencies ("*") in this crate's Cargo.toml to the highest version found
     * in `lock_versions`, or, if the dependency is a path-based neighbor crate and it has no version
     * (or has "*"), pins it to that neighbor crate's actual version.
     *
     * - If a dependency has a `path` but no `version`, we read that neighbor crate's version and set it.
     * - If a dependency has `version = "*"`, we first check if it has a `path`:
     *       - If so, use the neighbor crate's version.
     *       - Otherwise, look up the crate in the provided `lock_versions`.
     *         If found, use the highest version. If missing, leave as "*".
     * - If multiple distinct versions exist in `lock_versions`, picks the highest and logs a warning.
     * - Preserves formatting/comments as best as `toml_edit` allows.
     *
     * NOTE: This writes the updated TOML **in-place** to `self.path`.
     */
    async fn pin_wildcard_dependencies(
        &self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), Self::Error> {

        trace!("pin_wildcard_dependencies: reading original Cargo.toml from {:?}", self.as_ref());

        let original_toml_str = tokio::fs::read_to_string(self.as_ref())
            .await
            .map_err(|e| CargoTomlError::ReadError { io: e.into() })?;

        let mut doc = original_toml_str
            .parse::<toml_edit::Document>()
            .map_err(|parse_err| CargoTomlError::TomlEditError {
                cargo_toml_file: self.as_ref().to_path_buf(),
                toml_parse_error: parse_err,
            })?;

        trace!("pin_wildcard_dependencies: parsed Cargo.toml into toml_edit Document successfully.");

        // Perform in-place pinning
        pin_wildcards_in_doc(&mut doc, lock_versions, self).await?;

        let pinned_str = doc.to_string();
        debug!("pin_wildcard_dependencies: writing pinned TOML back to {:?}", self.as_ref());

        tokio::fs::write(self.as_ref(), pinned_str)
            .await
            .map_err(|e| CargoTomlWriteError::WriteError {
                io: e.into(),
                cargo_toml_file: self.as_ref().to_path_buf(),
            })?;

        info!("pin_wildcard_dependencies: successfully updated {:?}", self.as_ref());
        Ok(())
    }
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
    #[traced_test]
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
    #[traced_test]
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
    #[traced_test]
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
    #[traced_test]
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
    #[traced_test]
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
    #[traced_test]
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
