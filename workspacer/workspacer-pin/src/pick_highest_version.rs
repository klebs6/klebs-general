crate::ix!();

/// Picks the highest version from `lock_map` for `crate_name`. If multiple distinct versions exist,
/// warns and picks the highest. Returns None if not found at all.
pub fn pick_highest_version(
    crate_name: &str,
    lock_map: &BTreeMap<String, BTreeSet<Version>>,
) -> Option<String> {
    let set = lock_map.get(crate_name)?;
    if set.len() > 1 {
        warn!(
            "pick_highest_version: crate '{}' has multiple versions in Cargo.lock: {:?}. Using the highest.",
            crate_name, set
        );
    }
    let highest = set.iter().max().unwrap(); 
    Some(highest.to_string())
}

#[cfg(test)]
mod test_pick_highest_version_exhaustive {
    use super::*;

    /// A small helper to build a `BTreeMap<String,BTreeSet<Version>>`
    /// from a slice of tuples `(crate_name, versions)`.
    /// For example:
    ///   make_lock_map(&[("foo", &["1.2.3","2.0.0"]), ("bar", &["0.1.0"])])
    fn make_lock_map(data: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<Version>> {
        let mut map = BTreeMap::new();
        for (crate_name, versions) in data {
            let mut set = BTreeSet::new();
            for v in *versions {
                let parsed = Version::parse(v).expect("invalid semver in test data");
                set.insert(parsed);
            }
            map.insert(crate_name.to_string(), set);
        }
        map
    }

    #[traced_test]
    fn test_pick_highest_version_crate_not_in_map() {
        info!("Testing scenario where the requested crate is not in lock_map");
        let lock_map = make_lock_map(&[
            ("existing_crate", &["1.0.0"]),
        ]);

        let result = pick_highest_version("nonexistent_crate", &lock_map);
        assert!(result.is_none(), "Expected None if the crate is not in the map");
    }

    #[traced_test]
    fn test_pick_highest_version_single_version() {
        info!("Testing scenario with exactly one version in the set");
        let lock_map = make_lock_map(&[
            ("my_crate", &["1.2.3"]),
        ]);

        let result = pick_highest_version("my_crate", &lock_map);
        assert_eq!(result, Some("1.2.3".to_string()));
    }

    #[traced_test]
    fn test_pick_highest_version_multiple_versions() {
        info!("Testing scenario with multiple distinct versions in the set");
        // The function should pick the highest (2.1.0) and log a warning
        let lock_map = make_lock_map(&[
            ("my_crate", &["1.0.0", "2.1.0", "2.0.0"]),
        ]);

        let result = pick_highest_version("my_crate", &lock_map);
        assert_eq!(result, Some("2.1.0".to_string()), "Expected 2.1.0 as the highest");
        // We won't assert on the log message, but traced_test can capture it.
    }

    #[traced_test]
    fn test_pick_highest_version_multiple_crates_pick_correct_crate() {
        info!("Testing scenario with multiple crates in the map");
        let lock_map = make_lock_map(&[
            ("alpha", &["0.1.0", "0.2.5"]),
            ("beta", &["1.0.0"]),
            ("my_crate", &["1.4.3", "1.9.0", "1.8.7"]),
        ]);

        // We specifically request "my_crate"
        let result = pick_highest_version("my_crate", &lock_map);
        assert_eq!(result, Some("1.9.0".to_string()));
    }

    #[traced_test]
    fn test_pick_highest_version_semver_ordering() {
        info!("Test that we handle semver ordering (including prereleases, etc.)");
        let lock_map = make_lock_map(&[
            ("test_crate", &["1.2.3-alpha.1", "1.2.3-alpha.2", "1.2.3"]),
        ]);

        // The highest final version is "1.2.3" over any alpha/beta
        let result = pick_highest_version("test_crate", &lock_map);
        assert_eq!(result, Some("1.2.3".to_string()));
    }

    #[traced_test]
    fn test_pick_highest_version_same_versions() {
        info!("Testing scenario where the set contains duplicates (BTreeSet would unify them)");
        // BTreeSet automatically merges duplicates
        let lock_map = make_lock_map(&[
            ("duplicate_crate", &["1.0.0", "1.0.0", "1.0.0"]),
        ]);

        let result = pick_highest_version("duplicate_crate", &lock_map);
        assert_eq!(result, Some("1.0.0".to_string()));
        debug!("No warning should be logged because set.len() == 1 after duplicates unify");
    }
}
