crate::ix!();

/// Pins a wildcard string dependency (e.g. `serde = "*"`) from the lockfile if its version is `*`.
/// If no entry is found in the lockfile, logs a warning and leaves it as `*`.
pub async fn pin_wildcard_string_dependency(
    dep_item: &mut TeItem,
    dep_name: &str,
    lock_versions: &LockVersionMap,
) {
    trace!(
        "pin_wildcard_string_dependency: dep_name='{}'",
        dep_name
    );
    let current_str = dep_item.as_str().unwrap_or("");
    if current_str == "*" {
        debug!("pin_wildcard_string_dependency: found wildcard for '{}'", dep_name);
        pin_from_lock_or_warn(dep_item, dep_name, lock_versions);
    }
    trace!(
        "pin_wildcard_string_dependency: done dep_name='{}'",
        dep_name
    );
}

#[cfg(test)]
mod test_pin_wildcard_string_dependency {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn pins_star_using_lockfile() {
        info!("Starting test_pin_wildcard_string_dependency::pins_star_using_lockfile");

        // Arrange
        let mut item = TeItem::Value(TeValue::from("*"));
        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "somecrate".to_string(),
            vec![SemverVersion::parse("1.2.3").unwrap()]
                .into_iter()
                .collect(),
        );

        // Act
        pin_wildcard_string_dependency(&mut item, "somecrate", &lock_map).await;

        // Assert
        assert_eq!(item.as_str().unwrap(), "1.2.3");
        debug!("test_pin_wildcard_string_dependency::pins_star_using_lockfile passed");
    }
}
