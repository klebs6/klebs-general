crate::ix!();

/* =========================== 2) CHANGED ITEM ===========================
 *
 * Convert the cargo-lock `Version` to a string before creating a `TeValue`,
 * since `TomlEditValue: From<cargo_lock::Version>` isn't implemented.
 */
pub fn pin_from_lock_or_warn(
    dep_item: &mut TeItem,
    dep_name: &str,
    lock_versions: &LockVersionMap,
) {
    if let Some(new_ver) = pick_highest_version(dep_name, lock_versions) {
        let ver_str = new_ver.to_string();
        info!("Pinning wildcard '{}' => '{}'", dep_name, ver_str);
        match dep_item {
            TeItem::Value(TeValue::String { .. }) => {
                *dep_item = TeItem::Value(TeValue::from(ver_str));
            }
            TeItem::Value(TeValue::InlineTable(inline)) => {
                inline.insert("version", TeValue::from(ver_str));
            }
            TeItem::Table(tbl) => {
                tbl.insert("version", TeItem::Value(TeValue::from(ver_str)));
            }
            _ => {
                warn!("pin_from_lock_or_warn: item kind not recognized => cannot set version");
            }
        }
    } else {
        warn!(
            "pin_from_lock_or_warn: crate '{}' not in lock => leaving as '*'",
            dep_name
        );
    }
}

#[cfg(test)]
mod test_pin_from_lock_or_warn {
    use super::*;
    use tracing::{info, trace, debug};

    /// We test whether pin_from_lock_or_warn sets the version in-line from the lockfile.
    #[traced_test]
    async fn pins_inline_value() {
        info!("Starting test_pin_from_lock_or_warn::pins_inline_value");

        // Arrange
        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "examplecrate".to_string(),
            vec![
                SemverVersion::parse("0.1.0").unwrap(),
                SemverVersion::parse("0.2.5").unwrap(),
            ]
            .into_iter()
            .collect(),
        );

        let mut item = TeItem::Value(TeValue::from("*"));
        debug!("Before pinning: {:?}", item);

        // Act
        pin_from_lock_or_warn(&mut item, "examplecrate", &lock_map);

        // Assert
        match &item {
            TeItem::Value(TeValue::String { .. }) => {
                assert!(item.as_str().unwrap().starts_with("0.2.5"));
            }
            _ => panic!("pin_from_lock_or_warn did not convert the '*' into a string item"),
        }
        debug!("After pinning: {:?}", item);
    }

    /// We test whether pin_from_lock_or_warn warns if there is no matching crate in the lockfile.
    #[traced_test]
    async fn warns_when_not_in_lock() {
        info!("Starting test_pin_from_lock_or_warn::warns_when_not_in_lock");

        // Arrange
        let lock_map = BTreeMap::new(); // empty
        let mut item = TeItem::Value(TeValue::from("*"));

        // Act
        pin_from_lock_or_warn(&mut item, "missingcrate", &lock_map);

        // Assert
        // Should remain '*'
        assert_eq!(item.as_str().unwrap(), "*");
        debug!("Completed test with item still = {:?}", item);
    }
}

