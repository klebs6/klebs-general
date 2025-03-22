crate::ix!();

/// Recursively walk the TOML structure, looking for any sub-table named "*dependencies".
pub async fn fix_nested_tables(
    item: &mut toml_edit::Item,
    lock_versions: &LockVersionMap,
    root_cargo_toml: &CargoToml,
) -> Result<(), CargoTomlError> {
    if let toml_edit::Item::Table(tbl) = item {
        for (sub_key, sub_item) in tbl.iter_mut() {
            if is_dependencies_key(&sub_key) {
                if let toml_edit::Item::Table(dep_table) = sub_item {
                    pin_deps_in_table(dep_table, lock_versions, root_cargo_toml).await?;
                }
            }
            fix_nested_tables(sub_item, lock_versions, root_cargo_toml).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_fix_nested_tables_exhaustive {
    use super::*;

    // A mock or minimal synchronous approach to constructing toml_edit::Item structures
    // for testing fix_nested_tables. We'll define multiple test scenarios to ensure
    // comprehensive coverage of nested dependencies structures.

    #[traced_test]
    async fn test_fix_nested_tables_no_subtables() {
        info!("Starting test_fix_nested_tables_no_subtables...");

        // No dependencies anywhere
        let mut doc = Document::new();
        doc["package"] = toml_edit::value("some_package");
        doc["some_other_table"] = toml_edit::table();

        let mut item = Item::Table(doc.as_table_mut().clone());

        let lock_map: LockVersionMap = BTreeMap::new(); // empty, doesn't matter
        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok(), "Expected no error for a doc without any subtable named '*dependencies'");

        trace!("Completed test_fix_nested_tables_no_subtables.");
    }

    #[traced_test]
    async fn test_fix_nested_tables_single_level() {
        info!("Starting test_fix_nested_tables_single_level...");

        // A single table named [dependencies] but not nested
        // fix_nested_tables should still detect it if it's a sub-table in the root item
        // or if we nest it artificially inside some other table
        let mut doc = Document::new();

        let mut deps_tbl = Table::new();
        // Just a placeholder
        deps_tbl.insert("foo", toml_edit::value("*"));
        doc["dependencies"] = Item::Table(deps_tbl);

        let mut item = Item::Table(doc.as_table_mut().clone());

        // Provide a lock map that includes "foo"
        let lock_map: LockVersionMap = make_fake_lock_version_map(&[("foo", &["1.2.3"])]);

        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok(), "Should succeed pinning single-level dependencies subtable");

        // Let's see if the item was updated from '*' to '1.2.3'
        if let Item::Table(tbl) = &item {
            if let Some(Item::Table(deps)) = tbl.get("dependencies") {
                if let Some(foo_item) = deps.get("foo") {
                    let pinned = foo_item.as_str().unwrap_or("");
                    assert_eq!(pinned, "1.2.3", "Expected wildcard pinned to 1.2.3");
                } else {
                    panic!("Missing 'foo' key in pinned dependencies table");
                }
            } else {
                panic!("No 'dependencies' table found after fix_nested_tables");
            }
        } else {
            panic!("Root item isn't a table after fix_nested_tables");
        }

        trace!("Completed test_fix_nested_tables_single_level.");
    }

    #[traced_test]
    async fn test_fix_nested_tables_recursive_depth() {
        info!("Starting test_fix_nested_tables_recursive_depth...");

        // A scenario with nested structure:
        //
        //   [some_config]
        //   [some_config.sub_config]
        //   [some_config.sub_config.dependencies]
        //     bar = "*"
        //
        // fix_nested_tables should find that nested dependencies and attempt to pin it.
        let mut doc = Document::new();

        let mut nested_deps = Table::new();
        nested_deps.insert("bar", toml_edit::value("*"));

        let mut sub_config = Table::new();
        sub_config.insert("dependencies", Item::Table(nested_deps));

        let mut top_config = Table::new();
        top_config.insert("sub_config", Item::Table(sub_config));

        doc["some_config"] = Item::Table(top_config);

        let mut item = Item::Table(doc.as_table_mut().clone());

        // Provide a lock map with bar => "0.5.0"
        let lock_map = make_fake_lock_version_map(&[("bar", &["0.5.0"])]);

        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok());

        // Confirm pinned
        if let Item::Table(tbl) = &item {
            if let Some(Item::Table(some_config_tbl)) = tbl.get("some_config") {
                if let Some(Item::Table(sub_config_tbl)) = some_config_tbl.get("sub_config") {
                    if let Some(Item::Table(deps_tbl)) = sub_config_tbl.get("dependencies") {
                        let bar_item = deps_tbl.get("bar").expect("Expected 'bar' entry");
                        let pinned = bar_item.as_str().unwrap();
                        assert_eq!(pinned, "0.5.0", "Expected bar pinned to 0.5.0");
                    } else {
                        panic!("Missing nested dependencies table");
                    }
                } else {
                    panic!("Missing sub_config table");
                }
            } else {
                panic!("Missing some_config table");
            }
        } else {
            panic!("Root item not a table?");
        }

        trace!("Completed test_fix_nested_tables_recursive_depth.");
    }

    #[traced_test]
    async fn test_fix_nested_tables_multiple_dependency_tables() {
        info!("Starting test_fix_nested_tables_multiple_dependency_tables...");

        // This scenario has multiple sub-config sections, each containing a "dev-dependencies" or "build-dependencies"
        // or "dependencies" table. We want to ensure all are found and pinned.
        let mut doc = Document::new();

        let mut dev_deps_tbl = Table::new();
        dev_deps_tbl.insert("dev_thing", toml_edit::value("*"));

        let mut build_deps_tbl = Table::new();
        build_deps_tbl.insert("build_thing", toml_edit::value("*"));

        let mut sub_config_a = Table::new();
        sub_config_a.insert("dev-dependencies", Item::Table(dev_deps_tbl));

        let mut sub_config_b = Table::new();
        sub_config_b.insert("build-dependencies", Item::Table(build_deps_tbl));

        let mut top_config = Table::new();
        top_config.insert("sub_a", Item::Table(sub_config_a));
        top_config.insert("sub_b", Item::Table(sub_config_b));

        doc["some_root"] = Item::Table(top_config);

        let mut item = Item::Table(doc.as_table_mut().clone());

        // Provide lock map
        let lock_map = make_fake_lock_version_map(&[
            ("dev_thing", &["2.1.0"]),
            ("build_thing", &["3.5.7"]),
        ]);

        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok(), "Expected success pinning multiple sub-dependencies tables");

        if let Item::Table(tbl) = &item {
            if let Some(Item::Table(some_root_tbl)) = tbl.get("some_root") {
                // sub_a => dev-dependencies
                if let Some(Item::Table(sub_a_tbl)) = some_root_tbl.get("sub_a") {
                    if let Some(Item::Table(dev_tbl)) = sub_a_tbl.get("dev-dependencies") {
                        let pinned = dev_tbl.get("dev_thing").unwrap().as_str().unwrap();
                        assert_eq!(pinned, "2.1.0");
                    } else {
                        panic!("missing dev-dependencies in sub_a");
                    }
                } else {
                    panic!("missing sub_a in some_root");
                }

                // sub_b => build-dependencies
                if let Some(Item::Table(sub_b_tbl)) = some_root_tbl.get("sub_b") {
                    if let Some(Item::Table(build_tbl)) = sub_b_tbl.get("build-dependencies") {
                        let pinned = build_tbl.get("build_thing").unwrap().as_str().unwrap();
                        assert_eq!(pinned, "3.5.7");
                    } else {
                        panic!("missing build-dependencies in sub_b");
                    }
                } else {
                    panic!("missing sub_b in some_root");
                }
            } else {
                panic!("No some_root section found");
            }
        } else {
            panic!("root item not a table");
        }

        trace!("Completed test_fix_nested_tables_multiple_dependency_tables.");
    }

    #[traced_test]
    async fn test_fix_nested_tables_non_table_items_ignored() {
        info!("Starting test_fix_nested_tables_non_table_items_ignored...");

        // If we have a subkey named "dependencies" but it's not a Table, it should skip pin_deps_in_table call.
        let mut doc = Document::new();

        doc["foo"] = toml_edit::value("bar");
        doc["dependencies"] = toml_edit::value("this is not a table");
        // Or we have [some_config.dependencies], but that value is just a string
        let mut sc = Table::new();
        sc.insert("dependencies", toml_edit::value("string_value"));
        doc["some_config"] = Item::Table(sc);

        let mut item = Item::Table(doc.as_table_mut().clone());

        let lock_map = BTreeMap::new(); // doesn't matter
        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok(), "Should simply skip non-table 'dependencies' items with no errors");

        // Confirm everything is unchanged
        if let Item::Table(tbl) = &item {
            assert_eq!(tbl.get("dependencies").unwrap().as_str().unwrap(), "this is not a table");
            if let Some(Item::Table(some_config_tbl)) = tbl.get("some_config") {
                let val = some_config_tbl.get("dependencies").unwrap().as_str().unwrap();
                assert_eq!(val, "string_value");
            } else {
                panic!("Missing some_config table?");
            }
        } else {
            panic!("root item not a table");
        }

        trace!("Completed test_fix_nested_tables_non_table_items_ignored.");
    }

    #[traced_test]
    async fn test_fix_nested_tables_empty_table() {
        info!("Starting test_fix_nested_tables_empty_table...");

        // If there's an empty table named "dependencies", there's nothing to pin, but no error.
        let mut doc = Document::new();

        let empty_deps = Table::new();
        doc["nested"] = Item::Table(Table::new());
        doc["nested"]["dependencies"] = Item::Table(empty_deps);

        let mut item = Item::Table(doc.as_table_mut().clone());

        let lock_map = BTreeMap::new(); // no entries
        let cargo_toml = make_fake_cargo_toml_for_test();

        let result = fix_nested_tables(&mut item, &lock_map, &cargo_toml);
        assert!(result.is_ok(), "Empty table named 'dependencies' should not fail");

        // Confirm it remains empty
        if let Item::Table(tbl) = &item {
            if let Some(Item::Table(nested_tbl)) = tbl.get("nested") {
                if let Some(Item::Table(deps_tbl)) = nested_tbl.get("dependencies") {
                    assert!(deps_tbl.iter().next().is_none(), "Expected empty deps table");
                } else {
                    panic!("Missing nested.dependencies table");
                }
            } else {
                panic!("Missing nested table");
            }
        } else {
            panic!("root item not a table");
        }

        trace!("Completed test_fix_nested_tables_empty_table.");
    }

    /// Creates a fake LockVersionMap from array of (crate_name, [versions]).
    /// Example usage: make_fake_lock_version_map(&[("foo", &["1.2.3"])] )
    fn make_fake_lock_version_map(
        data: &[(&str, &[&str])]
    ) -> LockVersionMap {
        let mut map = BTreeMap::new();
        for (name, vers_array) in data {
            let mut set = BTreeSet::new();
            for v in *vers_array {
                let sem = Version::parse(v).unwrap();
                set.insert(sem);
            }
            map.insert(name.to_string(), set);
        }
        map
    }

    /// Minimal helper to create a "fake" CargoToml that references a path but won't be read.
    /// For these tests, we won't typically load from disk, so we can just provide a dummy path.
    fn make_fake_cargo_toml_for_test() -> CargoToml {
        // We can craft a new CargoToml with a dummy path. Implementation depends on your actual builder usage.
        CargoTomlBuilder::default()
            .path("dummy_path/Cargo.toml".into())
            .content(toml_edit::Value::Table(toml_edit::Table::new().clone()))
            .build()
            .expect("Failed to build a mock CargoToml for testing")
    }
}
