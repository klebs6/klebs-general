crate::ix!();

/// Pins a wildcard sub-table dependency (e.g. `[dependencies.somecrate] version="*", path="..."`).
/// If the dependency has a local path, uses [`GetVersionOfLocalDep`] to retrieve the version.
/// Otherwise, falls back to the lockfile.
pub async fn pin_wildcard_table_dependency(
    dep_item: &mut TeItem,
    dep_name: &str,
    lock_versions: &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) {
    trace!(
        "pin_wildcard_table_dependency: dep_name='{}'",
        dep_name
    );
    if let TeItem::Table(sub_table) = dep_item {
        let path_item = sub_table.get("path");
        let version_item = sub_table.get("version");

        if let Some(path_val) = path_item.and_then(|v| v.as_str()) {
            debug!(
                "pin_wildcard_table_dependency: local dep='{}', path='{}'",
                dep_name, path_val
            );
            if version_item.is_none() {
                if let Some(local_ver) =
                    root_cargo_toml.version_of_local_dep(dep_name, path_val).await
                {
                    info!(
                        "pin_wildcard_table_dependency: inserting local version='{}' for '{}'",
                        local_ver, dep_name
                    );
                    sub_table.insert("version", TeItem::Value(TeValue::from(local_ver)));
                }
            } else if let Some(v_str) = version_item.and_then(|x| x.as_str()) {
                if v_str == "*" {
                    debug!(
                        "pin_wildcard_table_dependency: wildcard version found for '{}'",
                        dep_name
                    );
                    if let Some(local_ver) =
                        root_cargo_toml.version_of_local_dep(dep_name, path_val).await
                    {
                        info!(
                            "pin_wildcard_table_dependency: inserting local version='{}' for '{}'",
                            local_ver, dep_name
                        );
                        sub_table.insert("version", TeItem::Value(TeValue::from(local_ver)));
                    }
                }
            }
        } else {
            // no `path`, so fall back to lockfile if version is "*"
            if let Some(v_str) = version_item.and_then(|x| x.as_str()) {
                if v_str == "*" {
                    debug!(
                        "pin_wildcard_table_dependency: wildcard version found for '{}'",
                        dep_name
                    );
                    pin_from_lock_or_warn(dep_item, dep_name, lock_versions);
                }
            }
        }
    }
    trace!(
        "pin_wildcard_table_dependency: done dep_name='{}'",
        dep_name
    );
}

#[cfg(test)]
mod test_pin_wildcard_table_dependency {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn inserts_local_version_if_none() {
        info!("Starting test_pin_wildcard_table_dependency::inserts_local_version_if_none");

        // Arrange
        let mut table = TeTable::default();
        table.insert("path", TeItem::Value(TeValue::from("some/local/dep")));
        // no version key
        let mut item = TeItem::Table(table);

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                Some("3.1.4".into())
            }
        }

        let lock_map = BTreeMap::new();

        // Act
        pin_wildcard_table_dependency(
            &mut item,
            "testdep",
            &lock_map,
            &MockGetVersion,
        ).await;

        // Assert
        if let TeItem::Table(t) = &item {
            let ver = t.get("version").unwrap().as_value().unwrap().as_str().unwrap();
            assert_eq!(ver, "3.1.4");
        } else {
            panic!("Expected a table item with version inserted");
        }
        debug!("test_pin_wildcard_table_dependency::inserts_local_version_if_none passed");
    }
}
