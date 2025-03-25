crate::ix!();

// A small helper to pick the highest version from lock map (already stable).
pub fn pick_highest_version(
    crate_name: &str,
    lock_map: &BTreeMap<String, BTreeSet<SemverVersion>>,
) -> Option<SemverVersion> {
    let set = lock_map.get(crate_name)?;
    if set.len() > 1 {
        warn!(
            "pick_highest_version: crate '{}' has multiple versions in lock map: {:?}. Using the highest.",
            crate_name, set
        );
    }
    let highest = set.iter().max().unwrap();
    Some(highest.clone())
}

#[cfg(test)]
mod test_pick_highest_version {
    use super::*;

    #[derive(Getters, Default)]
    struct DummyLockMap {
        #[getset(get = "pub")]
        map: BTreeMap<String, BTreeSet<SemverVersion>>,
    }

    #[traced_test]
    async fn picks_correct_highest_version() {
        info!("Starting test_pick_highest_version::picks_correct_highest_version");

        // Arrange
        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "serde".to_string(),
            vec![
                SemverVersion::parse("1.0.0").unwrap(),
                SemverVersion::parse("1.0.104").unwrap(),
            ]
            .into_iter()
            .collect(),
        );
        let dummy = DummyLockMap { map: lock_map };

        // Act
        let picked = pick_highest_version("serde", dummy.map());

        // Assert
        assert_eq!(picked.unwrap().to_string(), "1.0.104");
        debug!("test_pick_highest_version passed");
    }

    #[traced_test]
    async fn returns_none_when_not_found() {
        info!("Starting test_pick_highest_version::returns_none_when_not_found");

        // Arrange
        let dummy = DummyLockMap::default();

        // Act
        let picked = pick_highest_version("nonexistent", dummy.map());

        // Assert
        assert!(picked.is_none());
        debug!("test_pick_highest_version passed (no entry case)");
    }
}
