// ---------------- [ File: workspacer-pin/src/fix_nested_tables.rs ]
crate::ix!();

/* =========================== 3) CHANGED ITEM ===========================
 *
 * Rename `pin_deps_in_table` calls to `pin_wildcard_dependencies_in_table`
 * in the BFS-based approach for nested dependency tables.
 */
pub async fn fix_nested_tables(
    root_item: &mut TeItem,
    lock_versions: &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) -> Result<(), CargoTomlError> {
    trace!("fix_nested_tables: start BFS over nested tables");
    let mut queue: std::collections::VecDeque<Vec<String>> = std::collections::VecDeque::new();
    queue.push_back(vec![]); // start at the top-level (root_item)

    while let Some(path) = queue.pop_front() {
        let maybe_item = get_item_mut_by_path(root_item, &path);
        let curr_item = match maybe_item {
            Some(i) => i,
            None => {
                continue; // Path might be invalid if something was removed or replaced
            }
        };

        // If it's a Table, gather subkeys and look for any "*dependencies" sub-table
        if let TeItem::Table(tbl) = curr_item {
            let subkeys: Vec<String> = tbl.iter().map(|(k, _)| k.to_string()).collect();

            // a) If subkey ends with "dependencies", pin them
            for subkey in &subkeys {
                if is_dependencies_key(subkey) {
                    if let Some(deps_item) = tbl.get_mut(subkey) {
                        if let TeItem::Table(dep_table) = deps_item {
                            pin_wildcard_dependencies_in_table(
                                dep_table,
                                lock_versions,
                                root_cargo_toml,
                            )
                            .await?;
                        }
                    }
                }
            }

            // b) Enqueue sub-items
            for subkey in &subkeys {
                let mut child_path = path.clone();
                child_path.push(subkey.clone());
                queue.push_back(child_path);
            }
        }
    }

    trace!("fix_nested_tables: end BFS");
    Ok(())
}



/**
 * A small helper to descend a `TeItem` by following a sequence of keys.
 * Returns `None` if any key does not exist or is not a Table.
 *
 * We only hold each intermediate as mutable for the moment we descend it,
 * never holding multiple &mut references at once.
 */
fn get_item_mut_by_path<'a>(
    root: &'a mut TeItem,
    path: &[String],
) -> Option<&'a mut TeItem> {
    let mut current: *mut TeItem = root as *mut TeItem; 
    // unsafe pointer approach to avoid repeated borrowing,
    // but we can do a safe approach if we carefully nest .get_mut calls.

    // We'll do this carefully with a safe approach:
    // If the path is empty => the root
    if path.is_empty() {
        return Some(root);
    }

    let mut curr_item: Option<&mut TeItem> = Some(root);
    for (i, key) in path.iter().enumerate() {
        let item = curr_item?;
        match item {
            TeItem::Table(tbl) => {
                // if the key doesn't exist => return None
                let sub = tbl.get_mut(key);
                if sub.is_none() {
                    return None;
                }
                curr_item = sub;
            }
            // If at any point it's not a Table, we can't descend further
            _ => {
                return None;
            }
        }
    }

    curr_item
}

#[cfg(test)]
mod test_fix_nested_tables {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn pins_nested_dependencies() {
        info!("Starting test_fix_nested_tables::pins_nested_dependencies");

        // Arrange
        let mut doc = TeItem::Table(TeTable::default());
        if let TeItem::Table(tbl) = &mut doc {
            // top-level
            let mut nested = TeTable::default();
            nested.insert("dependencies", TeItem::Table(TeTable::default()));

            if let Some(TeItem::Table(dep_tbl)) = nested.get_mut("dependencies") {
                dep_tbl.insert("nesteddep", TeItem::Value(TeValue::from("*")));
            }
            tbl.insert("some_subtable", TeItem::Table(nested));
        }

        let mut lock_map = BTreeMap::new();
        lock_map.insert(
            "nesteddep".to_string(),
            vec![SemverVersion::parse("2.0.0").unwrap()]
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
        fix_nested_tables(&mut doc, &lock_map, &MockGetVersion).await.unwrap();

        // Assert
        // Expect nesteddep => "2.0.0"
        if let TeItem::Table(tbl) = &doc {
            if let Some(TeItem::Table(sub)) = tbl.get("some_subtable") {
                if let Some(TeItem::Table(deps)) = sub.get("dependencies") {
                    let pinned = deps.get("nesteddep").unwrap().as_str().unwrap();
                    assert_eq!(pinned, "2.0.0");
                } else {
                    panic!("missing dependencies table");
                }
            } else {
                panic!("missing some_subtable");
            }
        } else {
            panic!("root is not a table");
        }
        debug!("test_fix_nested_tables::pins_nested_dependencies passed");
    }
}
