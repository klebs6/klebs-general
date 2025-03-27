// ---------------- [ File: workspacer-pin/src/pin_wildcard_dependencies_in_table.rs ]
crate::ix!();

/// Iterates over a TOML `TeTable` (i.e., `[dependencies]` or similar),
/// and pins any wildcard dependencies found. This ensures that workspace
/// dependencies do not remain `*` when building or releasing.
pub async fn pin_wildcard_dependencies_in_table(
    table:           &mut TeTable,
    lock_versions:   &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) -> Result<(), CargoTomlError> {
    trace!("pin_wildcard_dependencies_in_table: start");
    let dep_keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();

    for dep_name in dep_keys {
        let dep_item = table.get_mut(&dep_name).expect("key must exist");
        match dep_item {
            TeItem::Value(TeValue::String { .. }) => {
                pin_wildcard_string_dependency(dep_item, &dep_name, lock_versions).await;
            }
            TeItem::Value(TeValue::InlineTable(_)) => {
                pin_wildcard_inline_table_dependency(
                    dep_item,
                    &dep_name,
                    lock_versions,
                    root_cargo_toml,
                )
                .await;
            }
            TeItem::Table(_) => {
                pin_wildcard_table_dependency(
                    dep_item,
                    &dep_name,
                    lock_versions,
                    root_cargo_toml,
                )
                .await;
            }
            _ => {
                trace!(
                    "pin_wildcard_dependencies_in_table: skipping dep='{}' => not string/inline/table",
                    dep_name
                );
            }
        }
    }

    trace!("pin_wildcard_dependencies_in_table: end");
    Ok(())
}

#[cfg(test)]
mod test_pin_wildcard_dependencies_in_table {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn handles_string_and_table() {
        info!("Starting test_pin_wildcard_dependencies_in_table::handles_string_and_table");

        // Arrange
        let mut table = TeTable::default();
        table.insert("libfoo", TeItem::Value(TeValue::from("*")));
        let mut sub_table = TeTable::default();
        sub_table.insert("version", TeItem::Value(TeValue::from("*")));
        table.insert("libbar", TeItem::Table(sub_table));

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                None
            }
        }

        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "libfoo".to_string(),
            vec![SemverVersion::parse("1.2.3").unwrap()]
                .into_iter()
                .collect(),
        );
        lock_map.insert(
            "libbar".to_string(),
            vec![SemverVersion::parse("0.9.9").unwrap()]
                .into_iter()
                .collect(),
        );

        // Act
        pin_wildcard_dependencies_in_table(&mut table, &lock_map, &MockGetVersion).await.unwrap();

        // Assert
        // libfoo => "1.2.3", libbar => "0.9.9"
        match table.get("libfoo").unwrap() {
            TeItem::Value(TeValue::String { .. }) => {
                assert_eq!(table.get("libfoo").unwrap().as_str().unwrap(), "1.2.3");
            }
            _ => panic!("libfoo not pinned from lock map"),
        }

        match table.get("libbar").unwrap() {
            TeItem::Table(tbl) => {
                let ver = tbl.get("version").unwrap().as_value().unwrap().as_str().unwrap();
                assert_eq!(ver, "0.9.9");
            }
            _ => panic!("libbar not pinned from lock map"),
        }
        debug!("test_pin_wildcard_dependencies_in_table::handles_string_and_table passed");
    }
}
