// ---------------- [ File: workspacer-pin/src/pin_wildcards_in_doc.rs ]
crate::ix!();

pub async fn pin_wildcards_in_doc(
    doc:             &mut TeDocument,
    lock_versions:   &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) -> Result<(), CargoTomlError> {
    trace!("pin_wildcards_in_doc: start");

    // -----------------------------------------------------------------
    // 1) Top‑level handling, including the special `[workspace.*]` table
    // -----------------------------------------------------------------
    {
        let top_keys: Vec<String> =
            doc.as_table_mut().iter().map(|(k, _)| k.to_string()).collect();

        for key in &top_keys {
            trace!("pin_wildcards_in_doc: inspecting top‑level key '{}'", key);

            // SAFETY: key came from iterator ⇒ lookup must succeed
            let item = doc
                .as_table_mut()
                .get_mut(key)
                .expect("key disappeared between iteration and lookup");

            // 1‑a) `[dependencies]`, `[dev‑dependencies]`, …
            if is_dependencies_key(key) {
                if let TeItem::Table(dep_tbl) = item {
                    trace!(
                        "pin_wildcards_in_doc: pinning classic '*dependencies' table '{}'",
                        key
                    );
                    pin_wildcard_dependencies_in_table(dep_tbl, lock_versions, root_cargo_toml)
                        .await?;
                }
            }
            // 1‑b) `[workspace.dependencies]`, `[workspace.dev‑dependencies]`, …
            else if key == "workspace" {
                if let TeItem::Table(ws_tbl) = item {
                    trace!("pin_wildcards_in_doc: descending into [workspace]");

                    let ws_dep_keys: Vec<String> =
                        ws_tbl.iter().map(|(k, _)| k.to_string()).collect();

                    for sub_key in &ws_dep_keys {
                        trace!(
                            "pin_wildcards_in_doc: inspecting [workspace.{}]",
                            sub_key
                        );
                        if is_dependencies_key(sub_key) {
                            if let Some(TeItem::Table(dep_tbl)) = ws_tbl.get_mut(sub_key) {
                                trace!(
                                    "pin_wildcards_in_doc: pinning [workspace.{}] table",
                                    sub_key
                                );
                                pin_wildcard_dependencies_in_table(
                                    dep_tbl,
                                    lock_versions,
                                    root_cargo_toml,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // 2) Recurse through **all** nested tables
    // -----------------------------------------------------------------
    trace!("pin_wildcards_in_doc: recursing through nested tables");
    fix_nested_tables(doc.as_item_mut(), lock_versions, root_cargo_toml).await?;

    trace!("pin_wildcards_in_doc: end");
    Ok(())
}

#[cfg(test)]
mod test_pin_wildcards_in_doc {
    use super::*;

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

    /// New regression‑test: verifies that wildcard versions inside
    /// `[workspace.dependencies]` are also pinned.
    #[traced_test]
    async fn pins_workspace_dependencies() {
        info!("test: pins_workspace_dependencies");

        // ---------- Arrange ----------
        let mut doc = TeDocument::new();
        {
            let tbl = doc.as_table_mut();

            // [workspace.dependencies]
            let mut ws_tbl = TeTable::default();
            ws_tbl.insert("dependencies", TeItem::Table(TeTable::default()));
            if let Some(TeItem::Table(dep_tbl)) = ws_tbl.get_mut("dependencies") {
                dep_tbl.insert("serde", TeItem::Value(TeValue::from("*")));
            }
            tbl.insert("workspace", TeItem::Table(ws_tbl));
        }

        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "serde".to_string(),
            vec![SemverVersion::parse("1.0.197").unwrap()]
                .into_iter()
                .collect(),
        );

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(
                &self,
                _dep_name: &str,
                _dep_path: &str,
            ) -> Option<String> {
                None
            }
        }

        // ---------- Act ----------
        pin_wildcards_in_doc(&mut doc, &lock_map, &MockGetVersion)
            .await
            .unwrap();

        // ---------- Assert ----------
        {
            let tbl = doc.as_table();
            let serde_val = tbl["workspace"]["dependencies"]["serde"]
                .as_str()
                .unwrap();
            assert_eq!(serde_val, "1.0.197");
        }

        debug!("test_pin_wildcards_in_doc::pins_workspace_dependencies passed");
    }
}
