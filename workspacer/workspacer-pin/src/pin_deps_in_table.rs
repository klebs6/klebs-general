crate::ix!();

/// Pin dependencies within a single `[dependencies]` table or `[dev-dependencies]` table.
pub async fn pin_deps_in_table(
    table: &mut toml_edit::Table,
    lock_versions: &LockVersionMap,
    root_cargo_toml: &CargoToml,
) -> Result<(), CargoTomlError> {
    // We collect the keys to operate on them more safely while iterating
    let dep_keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();

    for dep_name in dep_keys {
        let dep_item = table.get_mut(&dep_name).expect("key must exist");
        match dep_item {
            // Example: `serde_json = "*"` or `foo = "1.2.3"`
            toml_edit::Item::Value(toml_edit::Value::String { .. }) => {
                let current_str = dep_item.as_str().unwrap_or("");
                if current_str == "*" {
                    // If there's a wildcard, first check if there's a path field hidden somewhere? (Not typical for plain strings, but let's be safe.)
                    // Actually, for a plain string assignment, there's no place to store a path. So we just do lock-based pin.
                    pin_from_lock_or_warn(dep_item, &dep_name, lock_versions);
                }
                // If it's some other version string, we skip. Not a wildcard.
            }

            // Example: `serde = { version = "*", features=["derive"] }`
            toml_edit::Item::Value(toml_edit::Value::InlineTable(inline_tab)) => {
                let version_item = inline_tab.get("version");
                let path_item = inline_tab.get("path");

                // If we have a path, try to ensure version is pinned to neighbor crate's version
                if let Some(path_val) = path_item.and_then(|p| p.as_str()) {
                    // If there's no version key at all
                    if version_item.is_none() {
                        debug!(
                            "Found path-based dep '{}' with no version. Attempting to read local crate.",
                            dep_name
                        );
                        if let Some(local_ver) = root_cargo_toml.version_of_local_dep(&dep_name, path_val).await {
                            info!("Pinning path-based dep '{}' to '{}'", dep_name, local_ver);
                            inline_tab.insert("version", toml_edit::Value::from(local_ver));
                        } else {
                            warn!("Could not determine local version for path-based dep '{}'; leaving it unversioned.", dep_name);
                        }
                    }
                    // If there's version = "*" or version = X
                    else if let Some(ver_str) = version_item.and_then(|v| v.as_str()) {
                        if ver_str == "*" {
                            debug!(
                                "Found path-based dep '{}' with version='*'; pinning to local crate version.",
                                dep_name
                            );
                            if let Some(local_ver) = root_cargo_toml.version_of_local_dep(&dep_name, path_val).await {
                                info!(
                                    "Pinning path-based wildcard dep '{}' from '*' to '{}'",
                                    dep_name, local_ver
                                );
                                inline_tab.insert("version", toml_edit::Value::from(local_ver));
                            } else {
                                warn!(
                                    "Failed to retrieve local version for path-based wildcard dep '{}', leaving as '*'",
                                    dep_name
                                );
                            }
                        }
                        // else if it's a real version or something else => skip
                    }
                } else {
                    // No path => see if version is "*"
                    if let Some(ver_str) = version_item.and_then(|v| v.as_str()) {
                        if ver_str == "*" {
                            // Then we go to the lock
                            pin_from_lock_or_warn(dep_item, &dep_name, lock_versions);
                        }
                    }
                }
            }

            // Example: 
            //   [dependencies.my_crate]
            //   path = "../my_crate"
            //   version = "*"
            toml_edit::Item::Table(sub_table) => {
                let path_item = sub_table.get("path");
                let version_item = sub_table.get("version");

                if let Some(path_val) = path_item.and_then(|v| v.as_str()) {
                    // If there's no version
                    if version_item.is_none() {
                        debug!(
                            "Found path-based dep '{}' as sub-table with no version. Attempting local crate read.",
                            dep_name
                        );
                        if let Some(local_ver) = root_cargo_toml.version_of_local_dep(&dep_name, path_val).await {
                            info!("Pinning path-based dep '{}' to '{}'", dep_name, local_ver);
                            sub_table.insert("version", toml_edit::Item::Value(toml_edit::Value::from(local_ver)));
                        } else {
                            warn!(
                                "Could not find local version for path-based sub-table dep '{}'; leaving unversioned.",
                                dep_name
                            );
                        }
                    }
                    // If there's version = "*"
                    else if let Some(ver_str) = version_item.and_then(|x| x.as_str()) {
                        if ver_str == "*" {
                            debug!(
                                "Found path-based sub-table dep '{}' with version='*'; pinning to local crate.",
                                dep_name
                            );
                            if let Some(local_ver) = root_cargo_toml.version_of_local_dep(&dep_name, path_val).await {
                                info!(
                                    "Pinning wildcard sub-table dep '{}' from '*' to '{}'",
                                    dep_name, local_ver
                                );
                                sub_table.insert("version", toml_edit::Item::Value(toml_edit::Value::from(local_ver)));
                            } else {
                                warn!(
                                    "Could not find local version for path-based sub-table dep '{}', leaving as '*'.",
                                    dep_name
                                );
                            }
                        }
                    }
                } else {
                    // no path => if there's version="*", pin from lock
                    if let Some(version_val) = version_item {
                        if version_val.as_str() == Some("*") {
                            pin_from_lock_or_warn(dep_item, &dep_name, lock_versions);
                        }
                    }
                }
            }

            _ => {
                // Not a typical dependency expression, ignoring
                trace!(
                    "pin_deps_in_table: skipping dep '{}' because it's not a string, inline table, or table.",
                    dep_name
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_pin_deps_in_table_exhaustive {
    use super::*;
    use toml_edit::{Table,Value};

    // --------------------------------------------------------------------------------------------
    // A fake CargoToml for testing that always returns a fixed version for local dependencies,
    // or None if you choose. This avoids needing actual file I/O for local path crates.
    // --------------------------------------------------------------------------------------------
    #[derive(Clone, Debug)]
    struct FakeCargoToml {
        pub local_version: Option<String>,
    }

    #[async_trait]
    impl GetVersionOfLocalDep for FakeCargoToml {
        /// For testing, if `local_version` is Some, we return that. If it's None, we return None.
        async fn version_of_local_dep(&self, dep_name: &str, _dep_path: &str) -> Option<String> {
            debug!(
                "FakeCargoToml::version_of_local_dep called for '{}', returning {:?}",
                dep_name, self.local_version
            );
            self.local_version.clone()
        }
    }

    // We define a minimal function that instantiates a toml_edit::Table from pairs of
    // (dep_name, item). The `item` can be a string value, inline table, or sub-table,
    // precisely to simulate the conditions tested in pin_deps_in_table.
    fn make_dependencies_table(deps: Vec<(&str, Item)>) -> Table {
        let mut tbl = Table::new();
        for (dep_name, item) in deps {
            tbl.insert(dep_name, item);
        }
        tbl
    }

    /// Helper to build a LockVersionMap from an array of (crate_name, &["X.Y.Z", ...]).
    fn make_lock_map(data: &[(&str, &[&str])]) -> LockVersionMap {
        let mut map = BTreeMap::new();
        for (cname, vers_list) in data {
            let mut set = BTreeSet::new();
            for ver_str in *vers_list {
                let v = Version::parse(ver_str).unwrap();
                set.insert(v);
            }
            map.insert(cname.to_string(), set);
        }
        map
    }

    // We'll run each scenario in separate tests for clarity. Each test constructs a table
    // that triggers the relevant branch in `pin_deps_in_table`.

    // 1) If the dep is a simple string = "*", we pin from lock
    #[traced_test]
    async fn test_string_wildcard_pinned_from_lock() {
        let lock_map = make_lock_map(&[("serde_json", &["1.2.3"])]);

        // Our "root" cargo doesn't matter for path-based logic here, so we can choose any default
        let fake_ctoml = FakeCargoToml { local_version: None };

        // E.g. `serde_json = "*"`
        let mut tbl = make_dependencies_table(vec![
            ("serde_json", Item::Value(Value::from("*"))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Now we expect serde_json pinned to "1.2.3"
        let pinned_val = tbl.get("serde_json").unwrap().as_str().unwrap();
        assert_eq!(pinned_val, "1.2.3");
    }

    // 2) If the dep is a simple string with a non-wildcard version => no change
    #[traced_test]
    async fn test_string_non_wildcard_no_pin() {
        let lock_map = make_lock_map(&[("serde_json", &["1.2.3"])]);

        let fake_ctoml = FakeCargoToml { local_version: None };

        // E.g. `serde_json = "0.9.0"`
        let mut tbl = make_dependencies_table(vec![
            ("serde_json", Item::Value(Value::from("0.9.0"))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Should remain "0.9.0"
        let pinned_val = tbl.get("serde_json").unwrap().as_str().unwrap();
        assert_eq!(pinned_val, "0.9.0", "Expected no change for non-wildcard version");
    }

    // 3) Inline table with path + no version => we try local crate. If local_version is Some(...) => pinned.
    #[traced_test]
    async fn test_inline_table_path_no_version() {
        // lock map irrelevant for path-based
        let lock_map = BTreeMap::new();
        // We'll simulate a local crate returning version 9.8.7
        let fake_ctoml = FakeCargoToml { local_version: Some("9.8.7".to_string()) };

        // E.g. `serde = { path = "../serde" }`  (no "version" key)
        let mut inline = toml_edit::InlineTable::default();
        inline.insert("path", Value::from("../serde"));

        let mut tbl = make_dependencies_table(vec![
            ("serde", Item::Value(Value::InlineTable(inline))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Expect pinned to "9.8.7"
        if let Item::Value(Value::InlineTable(inline_tab)) = tbl.get("serde").unwrap() {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("9.8.7"));
        } else {
            panic!("serde not an inline table after pinning?");
        }
    }

    // 4) Inline table with path + version="*" => pinned from local crate
    #[traced_test]
    async fn test_inline_table_path_wildcard() {
        // We'll simulate local crate version= "2.1.1"
        let fake_ctoml = FakeCargoToml { local_version: Some("2.1.1".to_string()) };

        let lock_map = BTreeMap::new();

        // e.g. `dep = { path = "../dep", version="*"}`
        let mut inline = toml_edit::InlineTable::default();
        inline.insert("path", Value::from("../dep"));
        inline.insert("version", Value::from("*"));

        let mut tbl = make_dependencies_table(vec![
            ("my_dep", Item::Value(Value::InlineTable(inline))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Expect pinned to "2.1.1"
        if let Item::Value(Value::InlineTable(inline_tab)) = tbl.get("my_dep").unwrap() {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("2.1.1"));
        } else {
            panic!("my_dep not an inline table after pinning?");
        }
    }

    // 5) Inline table with path + version="some_non_wildcard" => no change
    #[traced_test]
    async fn test_inline_table_path_nonwildcard_no_change() {
        let fake_ctoml = FakeCargoToml { local_version: Some("2.0.0".to_string()) };
        let lock_map = BTreeMap::new();

        let mut inline = toml_edit::InlineTable::default();
        inline.insert("path", Value::from("../dep"));
        inline.insert("version", Value::from("0.1.2"));

        let mut tbl = make_dependencies_table(vec![
            ("my_dep", Item::Value(Value::InlineTable(inline))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Should remain "0.1.2"
        if let Item::Value(Value::InlineTable(inline_tab)) = tbl.get("my_dep").unwrap() {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("0.1.2"), "Expected no change");
        } else {
            panic!("my_dep not inline table?");
        }
    }

    // 6) Inline table with no path + version="*" => pinned from lock
    #[traced_test]
    async fn test_inline_table_no_path_wildcard() {
        let fake_ctoml = FakeCargoToml { local_version: None };
        // Suppose the lock says "some_crate" => "3.3.3"
        let lock_map = make_lock_map(&[("some_crate", &["3.3.3"])]);

        let mut inline = toml_edit::InlineTable::default();
        inline.insert("version", Value::from("*"));

        let mut tbl = make_dependencies_table(vec![
            ("some_crate", Item::Value(Value::InlineTable(inline))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Expect pinned to "3.3.3"
        if let Item::Value(Value::InlineTable(inline_tab)) = tbl.get("some_crate").unwrap() {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("3.3.3"));
        } else {
            panic!("some_crate not an inline table?");
        }
    }

    // 7) Inline table with no path + version="1.0.0" => no change
    #[traced_test]
    async fn test_inline_table_no_path_nonwildcard_no_change() {
        let fake_ctoml = FakeCargoToml { local_version: None };
        let lock_map = BTreeMap::new();

        let mut inline = toml_edit::InlineTable::default();
        inline.insert("version", Value::from("1.0.0"));

        let mut tbl = make_dependencies_table(vec![
            ("some_crate", Item::Value(Value::InlineTable(inline))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Should remain "1.0.0"
        if let Item::Value(Value::InlineTable(inline_tab)) = tbl.get("some_crate").unwrap() {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("1.0.0"));
        } else {
            panic!("some_crate not inline table");
        }
    }

    // 8) Table subitem with path + no version => pinned from local
    #[traced_test]
    async fn test_subtable_path_no_version() {
        let lock_map = BTreeMap::new();
        let fake_ctoml = FakeCargoToml { local_version: Some("9.9.9".to_string()) };

        // We'll build a sub-table:
        //   [my_dep]
        //   path = "../dep"
        let mut sub_table = Table::new();
        sub_table.insert("path", Item::Value(Value::from("../dep")));

        let mut tbl = make_dependencies_table(vec![
            ("my_dep", Item::Table(sub_table)),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Now we expect [my_dep].version = "9.9.9"
        if let Item::Table(t) = tbl.get("my_dep").unwrap() {
            let version_item = t.get("version");
            let pinned = version_item.and_then(|i| i.as_str());
            assert_eq!(pinned, Some("9.9.9"));
        } else {
            panic!("my_dep not a sub-table?");
        }
    }

    // 9) Table subitem with path + version="*" => pinned from local
    #[traced_test]
    async fn test_subtable_path_wildcard() {
        let fake_ctoml = FakeCargoToml { local_version: Some("8.0.0".to_string()) };
        let lock_map = BTreeMap::new();

        // [dep]
        // path = "../dep"
        // version = "*"
        let mut sub_table = Table::new();
        sub_table.insert("path", Item::Value(Value::from("../dep")));
        sub_table.insert("version", Item::Value(Value::from("*")));

        let mut tbl = make_dependencies_table(vec![
            ("dep", Item::Table(sub_table)),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // expect pinned to "8.0.0"
        if let Item::Table(t) = tbl.get("dep").unwrap() {
            let pinned = t.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("8.0.0"));
        } else {
            panic!("dep not a table?");
        }
    }

    // 10) Table subitem with path + version="some-other" => no change
    #[traced_test]
    async fn test_subtable_path_nonwildcard_no_change() {
        let fake_ctoml = FakeCargoToml { local_version: Some("999.999.999".to_string()) };
        let lock_map = BTreeMap::new();

        // [dep]
        // path = "../dep"
        // version = "0.5.0"
        let mut sub_table = Table::new();
        sub_table.insert("path", Item::Value(Value::from("../dep")));
        sub_table.insert("version", Item::Value(Value::from("0.5.0")));

        let mut tbl = make_dependencies_table(vec![
            ("dep", Item::Table(sub_table)),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Remains 0.5.0
        if let Item::Table(t) = tbl.get("dep").unwrap() {
            let pinned = t.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("0.5.0"), "Expected no change");
        } else {
            panic!("dep not a table?");
        }
    }

    // 11) Table subitem with no path => version="*" => pinned from lock
    #[traced_test]
    async fn test_subtable_no_path_wildcard() {
        let fake_ctoml = FakeCargoToml { local_version: None };
        let lock_map = make_lock_map(&[("my_lib", &["1.1.1"])]);

        // [my_lib]
        // version="*"
        let mut sub_table = Table::new();
        sub_table.insert("version", Item::Value(Value::from("*")));

        let mut tbl = make_dependencies_table(vec![
            ("my_lib", Item::Table(sub_table)),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        if let Item::Table(t) = tbl.get("my_lib").unwrap() {
            let pinned = t.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("1.1.1"), "Expected pinned from lock");
        } else {
            panic!("my_lib not a table?");
        }
    }

    // 12) Table subitem with no path => version="1.2.3" => no change
    #[traced_test]
    async fn test_subtable_no_path_nonwildcard_no_change() {
        let fake_ctoml = FakeCargoToml { local_version: None };
        let lock_map = make_lock_map(&[("my_lib", &["9.9.9"])]);

        let mut sub_table = Table::new();
        sub_table.insert("version", Item::Value(Value::from("1.2.3")));

        let mut tbl = make_dependencies_table(vec![
            ("my_lib", Item::Table(sub_table)),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // Should remain "1.2.3"
        if let Item::Table(t) = tbl.get("my_lib").unwrap() {
            let pinned = t.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("1.2.3"));
        } else {
            panic!("my_lib not a table?");
        }
    }

    // 13) If we have an item kind that doesn't match string, inline table, or table => skip
    #[traced_test]
    async fn test_skip_non_standard_items() {
        let fake_ctoml = FakeCargoToml { local_version: Some("9.9.9".to_string()) };
        let lock_map = make_lock_map(&[("dummy", &["1.0.0"])]);

        // e.g. "dummy = true" or "dummy = 123"
        let mut tbl = make_dependencies_table(vec![
            ("dep_bool", Item::Value(Value::from(true))),
            ("dep_int", Item::Value(Value::from(42))),
        ]);

        let result = pin_deps_in_table(&mut tbl, &lock_map, &fake_ctoml).await;
        assert!(result.is_ok());

        // We simply expect no changes
        assert_eq!(tbl.get("dep_bool").unwrap().as_bool(), Some(true));
        assert_eq!(tbl.get("dep_int").unwrap().as_integer(), Some(42));
    }
}
