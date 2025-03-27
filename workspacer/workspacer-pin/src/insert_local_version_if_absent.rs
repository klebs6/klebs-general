// ---------------- [ File: workspacer-pin/src/insert_local_version_if_absent.rs ]
crate::ix!();

/// If a local path is provided but no version is present, attempts to fetch the version
/// from the local Cargo.toml and inserts it.
pub async fn insert_local_version_if_absent(
    inline_tab: &mut TeInlineTable,
    dep_name: &str,
    path_val: &str,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) {
    trace!(
        "insert_local_version_if_absent: dep_name='{}', path='{}'",
        dep_name,
        path_val
    );
    if let Some(local_ver) = root_cargo_toml.version_of_local_dep(dep_name, path_val).await {
        info!(
            "insert_local_version_if_absent: inserting local version='{}' for '{}'",
            local_ver, dep_name
        );
        inline_tab.insert("version", TeValue::from(local_ver));
    } else {
        debug!(
            "insert_local_version_if_absent: no local version found for '{}'",
            dep_name
        );
    }
}

#[cfg(test)]
mod test_insert_local_version_if_absent {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn inserts_version_when_none() {
        info!("Starting test_insert_local_version_if_absent::inserts_version_when_none");

        // Arrange
        let mut inline = TeInlineTable::default();
        inline.insert("path", TeValue::from("some/dep/path"));
        let mut item = TeItem::Value(TeValue::InlineTable(inline));

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                Some("0.99.0".into())
            }
        }

        // Act
        if let TeItem::Value(TeValue::InlineTable(ref mut tab)) = item {
            insert_local_version_if_absent(tab, "testdep", "some/dep/path", &MockGetVersion).await;
        }

        // Assert
        if let TeItem::Value(TeValue::InlineTable(tab)) = &item {
            assert_eq!(tab.get("version").unwrap().as_str().unwrap(), "0.99.0");
        } else {
            panic!("Expected inline table with inserted version");
        }
        debug!("test_insert_local_version_if_absent::inserts_version_when_none passed");
    }
}
