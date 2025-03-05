// ---------------- [ File: workspacer-lock/src/build_lock_versions.rs ]
crate::ix!();

/// A shared helper that reads `Cargo.lock` from `root` and builds a map of crate->versions.
pub async fn build_lock_versions<P>(
    root: &P
) -> Result<BTreeMap<String, BTreeSet<cargo_lock::Version>>, CrateError>
where
    P: AsRef<Path> + Send + Sync,
{
    let lock_path = root.as_ref().join("Cargo.lock");
    if !lock_path.exists() {
        return Err(CrateError::FileNotFound {
            missing_file: lock_path,
        });
    }

    let lockfile_str = fs::read_to_string(&lock_path)
        .await
        .map_err(|e| CrateError::IoError {
            io_error: Arc::new(e),
            context:  format!("Failed to read Cargo.lock at {:?}", lock_path),
        })?;

    let lockfile = cargo_lock::Lockfile::from_str(&lockfile_str).map_err(|e| {
        CrateError::LockfileParseFailed {
            path: lock_path.clone(),
            message: format!("{e}"),
        }
    })?;

    let mut map: BTreeMap<String, BTreeSet<cargo_lock::Version>> = BTreeMap::new();
    for cargo_lock::Package { name, version, .. } in &lockfile.packages {
        map.entry(name.as_str().to_owned())
            .or_default()
            .insert(version.clone());
    }
    debug!("build_lock_versions: created map with {} crates", map.len());
    Ok(map)
}

#[cfg(test)]
mod test_build_lock_versions {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use tempfile::{tempdir, NamedTempFile};
    use tokio::fs::{create_dir, File};
    use tokio::io::AsyncWriteExt;
    use cargo_lock::Version;

    /// Convenience helper to write a Cargo.lock file in the provided directory,
    /// with user-specified string content.
    async fn write_cargo_lock_content<P: AsRef<Path>>(
        dir_path: P,
        content: &str,
    ) -> PathBuf {
        let cargo_lock_path = dir_path.as_ref().join("Cargo.lock");
        let mut file = File::create(&cargo_lock_path)
            .await
            .expect("Failed to create Cargo.lock file");
        file.write_all(content.as_bytes())
            .await
            .expect("Failed to write Cargo.lock content");
        cargo_lock_path
    }

    /// 1) Tests that `build_lock_versions` returns an error if `Cargo.lock` does not exist.
    #[tokio::test]
    async fn test_no_cargo_lock_file_returns_error() {
        // Create a temporary directory without a Cargo.lock
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let result = build_lock_versions(&tmp_dir.path()).await;

        assert!(result.is_err(), "Expected an error when Cargo.lock is missing");
        match result {
            Err(CrateError::FileNotFound { missing_file }) => {
                assert!(missing_file.ends_with("Cargo.lock"));
            }
            other => {
                panic!("Expected CrateError::FileNotFound, got {:?}", other);
            }
        }
    }

    /// 2) Tests that `build_lock_versions` returns `LockfileParseFailed` for a malformed Cargo.lock.
    #[tokio::test]
    async fn test_malformed_cargo_lock_returns_parse_error() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        // Write some nonsense content
        let bad_content = r#"
            THIS IS MALFORMED!
            Not valid TOML or valid cargo lock syntax
        "#;
        write_cargo_lock_content(tmp_dir.path(), bad_content).await;

        let result = build_lock_versions(&tmp_dir.path()).await;
        assert!(result.is_err(), "Expected an error for malformed Cargo.lock");
        match result {
            Err(CrateError::LockfileParseFailed { path, message }) => {
                assert!(path.ends_with("Cargo.lock"), "Expected path to end with Cargo.lock");
                assert!(!message.is_empty(), "Error message should not be empty");
            }
            other => panic!("Expected CrateError::LockfileParseFailed, got {:?}", other),
        }
    }

    /// 3) Tests that `build_lock_versions` returns `IoError` if Cargo.lock cannot be read.
    /// (On some platforms, removing read permissions might require OS-specific logic.
    ///  Another approach is to turn Cargo.lock into a directory, which should fail to read as a file.)
    #[tokio::test]
    async fn test_unreadable_cargo_lock_returns_io_error() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let cargo_lock_path = tmp_dir.path().join("Cargo.lock");

        // Create a directory named "Cargo.lock" so reading as a file fails
        create_dir(&cargo_lock_path).await.expect("Failed to create dir as Cargo.lock");

        let result = build_lock_versions(&tmp_dir.path()).await;
        assert!(result.is_err(), "Expected an error when Cargo.lock is unreadable");
        match result {
            Err(CrateError::IoError { context, .. }) => {
                assert!(
                    context.contains("Failed to read Cargo.lock"),
                    "Expected context message referencing read failure"
                );
            }
            other => {
                panic!("Expected CrateError::IoError, got {:?}", other);
            }
        }
    }

    /// 4) Tests successful parsing of a well-formed Cargo.lock, verifying that the returned
    /// BTreeMap has the correct crate names and versions.
    #[tokio::test]
    async fn test_valid_cargo_lock_multiple_versions() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        // Example Cargo.lock content with multiple entries for the same crate
        // plus a distinct crate. 
        // `cargo_lock` crate can parse something like:
        //
        //   [[package]]
        //   name = "foo"
        //   version = "0.1.0"
        //   source = "registry+https://github.com/rust-lang/crates.io-index"
        //
        //   [[package]]
        //   name = "bar"
        //   version = "1.2.3"
        //   source = "registry+https://github.com/rust-lang/crates.io-index"
        //
        //   [[package]]
        //   name = "foo"
        //   version = "0.2.0"
        //   source = "registry+https://github.com/rust-lang/crates.io-index"
        //
        let cargo_lock_content = r#"
            [[package]]
            name = "foo"
            version = "0.1.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"

            [[package]]
            name = "bar"
            version = "1.2.3"
            source = "registry+https://github.com/rust-lang/crates.io-index"

            [[package]]
            name = "foo"
            version = "0.2.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"
        "#;

        write_cargo_lock_content(&tmp_dir.path(), cargo_lock_content).await;
        let result = build_lock_versions(&tmp_dir.path()).await;
        assert!(result.is_ok(), "Expected successful parse of valid Cargo.lock");
        let map = result.unwrap();

        // Verify we have two entries: "bar" and "foo"
        assert_eq!(map.len(), 2, "Expected 2 distinct crate names in the map");

        // For "foo", we expect two distinct versions: 0.1.0 and 0.2.0
        let foo_versions = map.get("foo").expect("Expected 'foo' in the map");
        assert_eq!(foo_versions.len(), 2, "Expected 2 versions for crate 'foo'");
        assert!(foo_versions.contains(&Version::parse("0.1.0").unwrap()));
        assert!(foo_versions.contains(&Version::parse("0.2.0").unwrap()));

        // For "bar", we expect a single version: 1.2.3
        let bar_versions = map.get("bar").expect("Expected 'bar' in the map");
        assert_eq!(bar_versions.len(), 1, "Expected 1 version for crate 'bar'");
        assert!(bar_versions.contains(&Version::parse("1.2.3").unwrap()));
    }

    /// 5) Optional: Test a minimal valid Cargo.lock with just one crate to confirm single-entry scenario.
    #[tokio::test]
    async fn test_valid_cargo_lock_single_package() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");

        let cargo_lock_content = r#"
            [[package]]
            name = "single_crate"
            version = "0.3.1"
            source = "registry+https://github.com/rust-lang/crates.io-index"
        "#;

        write_cargo_lock_content(&tmp_dir.path(), cargo_lock_content).await;
        let result = build_lock_versions(&tmp_dir.path()).await;
        assert!(result.is_ok(), "Expected successful parse of valid single-package Cargo.lock");
        let map = result.unwrap();

        assert_eq!(map.len(), 1, "Expected exactly 1 crate in the map");

        // For "single_crate", we expect a single version: 0.3.1
        let single_versions = map
            .get("single_crate")
            .expect("Expected 'single_crate' in the map");
        assert_eq!(single_versions.len(), 1, "Expected 1 version for 'single_crate'");
        assert!(single_versions.contains(&Version::parse("0.3.1").unwrap()));
    }
}
