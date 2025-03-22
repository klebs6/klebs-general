crate::ix!();

/// If `dep_item` is a wildcard, look up `dep_name` in lock_versions and pin to highest. Otherwise warn.
pub fn pin_from_lock_or_warn(
    dep_item: &mut toml_edit::Item,
    dep_name: &str,
    lock_versions: &LockVersionMap,
) {
    if let Some(new_ver) = pick_highest_version(dep_name, lock_versions) {
        info!("Pinning wildcard dep '{}' from '*' to '{}'", dep_name, new_ver);
        match dep_item {
            toml_edit::Item::Value(toml_edit::Value::String { .. }) => {
                *dep_item = toml_edit::Item::Value(toml_edit::Value::from(new_ver));
            }
            toml_edit::Item::Value(toml_edit::Value::InlineTable(inline_tab)) => {
                inline_tab.insert("version", toml_edit::Value::from(new_ver));
            }
            toml_edit::Item::Table(tbl) => {
                tbl.insert("version", toml_edit::Item::Value(toml_edit::Value::from(new_ver)));
            }
            _ => {
                warn!(
                    "pin_from_lock_or_warn: got unexpected item kind for '{}', cannot set version properly.",
                    dep_name
                );
            }
        }
    } else {
        warn!(
            "pin_from_lock_or_warn: wildcard dep '{}' was not found in Cargo.lock; leaving as '*'",
            dep_name
        );
    }
}

#[cfg(test)]
mod test_pin_from_lock_or_warn_exhaustive {
    use super::*;

    /// A helper to build a LockVersionMap from an array of (crate_name, &["versions"...]).
    fn make_lock_map(data: &[(&str, &[&str])]) -> LockVersionMap {
        let mut map = BTreeMap::new();
        for (crate_name, vers_list) in data {
            let mut set = BTreeSet::new();
            for &ver_str in *vers_list {
                let sem = Version::parse(ver_str).expect("Invalid semver in test data");
                set.insert(sem);
            }
            map.insert(crate_name.to_string(), set);
        }
        map
    }

    // A small convenience function to create a toml_edit::Item::Value(String).
    fn make_string_item(s: &str) -> Item {
        Item::Value(Value::from(s))
    }

    // A convenience to create an InlineTable item with `version="*"`.
    fn make_inline_wildcard() -> Item {
        let mut inline = toml_edit::InlineTable::default();
        inline.insert("version", Value::from("*"));
        Item::Value(Value::InlineTable(inline))
    }

    // A convenience to create a sub-Table item with `version="*"`.
    fn make_subtable_wildcard() -> Item {
        let mut tbl = Table::new();
        tbl.insert("version", make_string_item("*"));
        Item::Table(tbl)
    }

    #[traced_test]
    fn test_no_version_found_in_lock_map() {
        info!("Testing scenario: wildcard dep not present in lock map => remains '*'");
        let lock_map = make_lock_map(&[
            ("another_crate", &["0.1.0"]),
        ]);
        let mut item = make_string_item("*");

        pin_from_lock_or_warn(&mut item, "nonexistent_dep", &lock_map);

        // Should remain '*', because nonexistent_dep isn't in lock_map
        assert_eq!(item.as_str().unwrap(), "*");
    }

    #[traced_test]
    fn test_string_wildcard_found_in_lock_map() {
        info!("Testing scenario: wildcard dep is a plain string, single version in lock => pinned");

        let lock_map = make_lock_map(&[
            ("serde", &["1.0.155"]),
        ]);
        let mut item = make_string_item("*");

        pin_from_lock_or_warn(&mut item, "serde", &lock_map);
        // Expect pinned
        assert_eq!(item.as_str().unwrap(), "1.0.155");
    }

    #[traced_test]
    fn test_string_wildcard_multiple_versions_in_lock() {
        info!("Testing scenario: wildcard dep is a plain string, multiple lock versions => picks highest");

        let lock_map = make_lock_map(&[
            ("serde_json", &["1.0.80", "1.1.0", "0.9.9"]),
        ]);
        let mut item = make_string_item("*");

        pin_from_lock_or_warn(&mut item, "serde_json", &lock_map);

        // Should pick "1.1.0" as highest
        assert_eq!(item.as_str().unwrap(), "1.1.0");
    }

    #[traced_test]
    fn test_inline_table_wildcard_pinned() {
        info!("Testing scenario: Item::Value(InlineTable) with version='*' => pinned from lock");
        let lock_map = make_lock_map(&[
            ("my_crate", &["2.0.0"]),
        ]);
        let mut inline_item = make_inline_wildcard();

        pin_from_lock_or_warn(&mut inline_item, "my_crate", &lock_map);

        // Now we expect the inline table to have version="2.0.0"
        if let Item::Value(Value::InlineTable(inline_tab)) = &inline_item {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("2.0.0"));
        } else {
            panic!("Expected Item::Value(InlineTable) after pinning, but got something else");
        }
    }

    #[traced_test]
    fn test_subtable_wildcard_pinned() {
        info!("Testing scenario: Item::Table with version='*' => pinned from lock");

        let lock_map = make_lock_map(&[
            ("sub_dep", &["3.3.3"]),
        ]);
        let mut sub_table_item = make_subtable_wildcard();

        pin_from_lock_or_warn(&mut sub_table_item, "sub_dep", &lock_map);

        if let Item::Table(tbl) = &sub_table_item {
            let pinned_val = tbl.get("version").and_then(|i| i.as_str());
            assert_eq!(pinned_val, Some("3.3.3"), "Expected pinned to 3.3.3");
        } else {
            panic!("Expected sub-table item after pinning, but got something else");
        }
    }

    #[traced_test]
    fn test_unexpected_item_kind() {
        info!("Testing scenario: item is neither string, inline table, nor sub-table => logs warning & does nothing");
        // For instance, an integer or boolean
        let lock_map = make_lock_map(&[
            ("int_dep", &["1.0.0"]),
        ]);

        let mut item = Item::Value(Value::from(true)); // Not a string, inline table, or table
        pin_from_lock_or_warn(&mut item, "int_dep", &lock_map);

        // The wildcard check doesn't even apply, so no changes
        // The item is still a boolean true
        assert_eq!(item.as_bool(), Some(true));
    }

    #[traced_test]
    fn test_no_wildcard_in_string() {
        info!("Testing scenario: item is a string but not wildcard => do nothing");
        let lock_map = make_lock_map(&[
            ("foo", &["1.0.0"]),
        ]);
        let mut item = make_string_item("0.9.0");

        pin_from_lock_or_warn(&mut item, "foo", &lock_map);

        // Should remain "0.9.0"
        assert_eq!(item.as_str(), Some("0.9.0"));
    }

    #[traced_test]
    fn test_no_wildcard_in_inline_table() {
        info!("Testing scenario: inline table with version='0.1.2' => do nothing");
        let lock_map = make_lock_map(&[("dep_a", &["0.2.0"])]);

        let mut inline = toml_edit::InlineTable::default();
        inline.insert("version", Value::from("0.1.2"));
        let mut item = Item::Value(Value::InlineTable(inline));

        pin_from_lock_or_warn(&mut item, "dep_a", &lock_map);

        // Expect no change
        if let Item::Value(Value::InlineTable(inline_tab)) = &item {
            let pinned = inline_tab.get("version").and_then(|v| v.as_str());
            assert_eq!(pinned, Some("0.1.2"));
        } else {
            panic!("Should remain inline table with version=0.1.2");
        }
    }

    #[traced_test]
    fn test_no_wildcard_in_subtable() {
        info!("Testing scenario: sub-table with version='1.2.3' => do nothing");
        let lock_map = make_lock_map(&[("dep_b", &["2.2.2"])]);

        let mut sub_tbl = Table::new();
        sub_tbl.insert("version", make_string_item("1.2.3"));
        let mut item = Item::Table(sub_tbl);

        pin_from_lock_or_warn(&mut item, "dep_b", &lock_map);

        // Expect no change
        if let Item::Table(t) = &item {
            let pinned = t.get("version").and_then(|i| i.as_str());
            assert_eq!(pinned, Some("1.2.3"));
        } else {
            panic!("Should remain table with version=1.2.3");
        }
    }
}
