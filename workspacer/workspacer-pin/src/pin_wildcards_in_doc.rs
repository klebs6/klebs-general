// ---------------- [ File: workspacer-pin/src/pin_wildcards_in_doc.rs ]
crate::ix!();

/* =========================== 4) CHANGED ITEM ===========================
 *
 * Rename `pin_deps_in_table` to `pin_wildcard_dependencies_in_table`
 * in the top-level function that processes a `TeDocument`.
 */
pub async fn pin_wildcards_in_doc(
    doc:             &mut TeDocument,
    lock_versions:   &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) -> Result<(), CargoTomlError> {
    // 1) top-level
    {
        let top_keys: Vec<String> = doc.as_table_mut().iter().map(|(k, _)| k.to_string()).collect();
        for key in top_keys {
            let item = doc.as_table_mut().get_mut(&key).unwrap();
            if is_dependencies_key(&key) {
                if let TeItem::Table(dep_tbl) = item {
                    pin_wildcard_dependencies_in_table(dep_tbl, lock_versions, root_cargo_toml)
                        .await?;
                }
            }
        }
    }

    // 2) nested
    fix_nested_tables(doc.as_item_mut(), lock_versions, root_cargo_toml).await?;
    Ok(())
}

#[cfg(test)]
mod test_pin_wildcards_in_doc {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn pins_top_level_and_nested() {
        info!("Starting test_pin_wildcards_in_doc::pins_top_level_and_nested");

        // Arrange
        let mut doc = TeDocument::new();
        {
            let tbl = doc.as_table_mut();
            tbl.insert("dependencies", TeItem::Table(TeTable::default()));
            if let Some(TeItem::Table(dep_tbl)) = tbl.get_mut("dependencies") {
                dep_tbl.insert("rand", TeItem::Value(TeValue::from("*")));
            }

            // nested
            let mut sub_tbl = TeTable::default();
            sub_tbl.insert("dev-dependencies", TeItem::Table(TeTable::default()));
            if let Some(TeItem::Table(dev_dep_tbl)) = sub_tbl.get_mut("dev-dependencies") {
                dev_dep_tbl.insert("regex", TeItem::Value(TeValue::from("*")));
            }
            tbl.insert("package", TeItem::Table(sub_tbl));
        }

        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "rand".to_string(),
            vec![SemverVersion::parse("0.8.5").unwrap()]
                .into_iter()
                .collect(),
        );
        lock_map.insert(
            "regex".to_string(),
            vec![SemverVersion::parse("1.7.3").unwrap()]
                .into_iter()
                .collect(),
        );

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                None
            }
        }

        // Act
        pin_wildcards_in_doc(&mut doc, &lock_map, &MockGetVersion).await.unwrap();

        // Assert
        {
            let tbl = doc.as_table();
            // rand pinned?
            let rand_val = tbl["dependencies"]["rand"].as_str().unwrap();
            assert_eq!(rand_val, "0.8.5");

            // regex pinned?
            let regex_val = tbl["package"]["dev-dependencies"]["regex"].as_str().unwrap();
            assert_eq!(regex_val, "1.7.3");
        }
        debug!("test_pin_wildcards_in_doc::pins_top_level_and_nested passed");
    }
}
