// ---------------- [ File: workspacer-pin/src/replace_wildcard_version_with_local.rs ]
crate::ix!();

/// If a local path is provided and the version is `"*"`, fetches the local version
/// and replaces the wildcard.
pub async fn replace_wildcard_version_with_local(
    inline_tab: &mut TeInlineTable,
    dep_name: &str,
    path_val: &str,
    root_cargo_toml: &dyn GetVersionOfLocalDep,
) {
    trace!(
        "replace_wildcard_version_with_local: dep_name='{}', path='{}'",
        dep_name,
        path_val
    );
    if let Some(local_ver) = root_cargo_toml.version_of_local_dep(dep_name, path_val).await {
        info!(
            "replace_wildcard_version_with_local: inserting local version='{}' for '{}'",
            local_ver, dep_name
        );
        inline_tab.insert("version", TeValue::from(local_ver));
    } else {
        debug!(
            "replace_wildcard_version_with_local: no local version found for '{}'",
            dep_name
        );
    }
}

#[cfg(test)]
mod test_replace_wildcard_version_with_local {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn replaces_star_version() {
        info!("Starting test_replace_wildcard_version_with_local::replaces_star_version");

        // Arrange
        let mut inline = TeInlineTable::default();
        inline.insert("path", TeValue::from("my/dep/path"));
        inline.insert("version", TeValue::from("*"));
        let mut item = TeItem::Value(TeValue::InlineTable(inline));

        struct MockGetVersion;
        #[async_trait]
        impl GetVersionOfLocalDep for MockGetVersion {
            async fn version_of_local_dep(&self, _dep_name: &str, _dep_path: &str) -> Option<String> {
                Some("9.8.7".into())
            }
        }

        // Act
        if let TeItem::Value(TeValue::InlineTable(ref mut tab)) = item {
            replace_wildcard_version_with_local(tab, "dummydep", "my/dep/path", &MockGetVersion).await;
        }

        // Assert
        if let TeItem::Value(TeValue::InlineTable(tab)) = &item {
            let ver = tab.get("version").unwrap().as_str().unwrap();
            assert_eq!(ver, "9.8.7");
        } else {
            panic!("Expected inline table with replaced version");
        }
        debug!("test_replace_wildcard_version_with_local::replaces_star_version passed");
    }
}
