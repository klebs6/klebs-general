crate::ix!();

/* =========================== 1) CHANGED ITEM ===========================
 *
 * Fix the E0502 error in `pin_wildcard_inline_table_dependency`
 * by not holding immutable references (to `inline_tab`) while
 * trying to mutate them. We do this by copying out the path/value
 * fields into locals first.
 */
pub async fn pin_wildcard_inline_table_dependency(
    dep_item: &mut TeItem,
    dep_name: &str,
    lock_versions: &LockVersionMap,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) {
    trace!(
        "pin_wildcard_inline_table_dependency: dep_name='{}'",
        dep_name
    );
    if let TeItem::Value(TeValue::InlineTable(inline_tab)) = dep_item {
        let path_val = inline_tab
            .get("path")
            .and_then(|p| p.as_str())
            .map(String::from);
        let version_val = inline_tab
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);

        match (path_val, version_val) {
            // If local path is present but no version item, insert local version if possible
            (Some(path_str), None) => {
                debug!(
                    "pin_wildcard_inline_table_dependency: local dep='{}', path='{}', no version present",
                    dep_name, path_str
                );
                insert_local_version_if_absent(inline_tab, dep_name, &path_str, root_cargo_toml).await;
            }
            // If local path is present and version is "*", replace wildcard with local version
            (Some(path_str), Some(v_str)) if v_str == "*" => {
                debug!(
                    "pin_wildcard_inline_table_dependency: local dep='{}', path='{}', wildcard version found",
                    dep_name, path_str
                );
                replace_wildcard_version_with_local(inline_tab, dep_name, &path_str, root_cargo_toml).await;
            }
            // If no path is present but version is "*", pin from lockfile
            (None, Some(v_str)) if v_str == "*" => {
                debug!(
                    "pin_wildcard_inline_table_dependency: dep='{}', no path, wildcard version found",
                    dep_name
                );
                pin_from_lock_or_warn(dep_item, dep_name, lock_versions);
            }
            // Otherwise, do nothing special
            _ => {
                trace!(
                    "pin_wildcard_inline_table_dependency: no wildcard action required for dep='{}'",
                    dep_name
                );
            }
        }
    }
    trace!(
        "pin_wildcard_inline_table_dependency: done dep_name='{}'",
        dep_name
    );
}

#[cfg(test)]
mod test_pin_wildcard_inline_table_dependency {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn pins_local_star_version() {
        info!("Starting test_pin_wildcard_inline_table_dependency::pins_local_star_version");

        // Arrange
        let mut inline = TeInlineTable::default();
        inline.insert("path", TeValue::from("some/local/path"));
        inline.insert("version", TeValue::from("*"));
        let mut item = TeItem::Value(TeValue::InlineTable(inline));

        // Provide a mock GetVersionOfLocalDep that always returns Some("2.0.1")
        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                Some("2.0.1".to_string())
            }
        }

        let lock_map = BTreeMap::new(); // We won't use it here

        // Act
        pin_wildcard_inline_table_dependency(
            &mut item,
            "mockcrate",
            &lock_map,
            &MockGetVersion,
        ).await;

        // Assert
        if let TeItem::Value(TeValue::InlineTable(t)) = &item {
            assert_eq!(t.get("version").unwrap().as_str().unwrap(), "2.0.1");
        } else {
            panic!("Expected an inline table with pinned version");
        }
        debug!("test_pin_wildcard_inline_table_dependency::pins_local_star_version passed");
    }
}

