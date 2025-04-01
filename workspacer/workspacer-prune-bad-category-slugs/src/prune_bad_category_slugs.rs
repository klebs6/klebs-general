crate::ix!();

#[async_trait]
pub trait PruneInvalidCategorySlugs {
    type Error;

    async fn prune_invalid_category_slugs(&mut self) -> Result<usize, Self::Error>;
}

#[async_trait]
impl<T> PruneInvalidCategorySlugs for T
where
    T: CrateHandleInterface<PathBuf> 
        + HasCargoToml
        + AsRef<Path>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    type Error = CrateError;

    async fn prune_invalid_category_slugs(&mut self) -> Result<usize, Self::Error> {
        trace!("(Single crate) begin: path={:?}", self.as_ref());

        let cargo_arc = self.cargo_toml();
        let mut cargo_guard = cargo_arc.lock().await;

        let mut doc = cargo_guard.document_clone().await?;
        let mut removed_count = 0_usize;

        let re = regex::Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();

        if let Some(package_item) = doc.get_mut("package") {
            if let Some(pkg_table) = package_item.as_table_mut() {
                // categories
                if let Some(cats_val) = pkg_table.get_mut("categories") {
                    if let Some(arr) = cats_val.as_array_mut() {
                        let mut i = 0;
                        while i < arr.len() {
                            if let Some(cat_str) = arr.get(i).and_then(|v| v.as_str()) {
                                let is_allowed = LEGAL_CATEGORIES.contains(&cat_str);
                                let matches_pattern = re.is_match(cat_str);
                                if !(is_allowed && matches_pattern) {
                                    debug!("Removing invalid category '{cat_str}'");
                                    arr.remove(i);
                                    removed_count += 1;
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                }
                // keywords
                if let Some(keys_val) = pkg_table.get_mut("keywords") {
                    if let Some(arr) = keys_val.as_array_mut() {
                        let mut i = 0;
                        while i < arr.len() {
                            if let Some(kw_str) = arr.get(i).and_then(|v| v.as_str()) {
                                if !re.is_match(kw_str) || kw_str.contains(' ') {
                                    debug!("Removing invalid keyword '{kw_str}'");
                                    arr.remove(i);
                                    removed_count += 1;
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                }
            }
        }

        cargo_guard.write_document_back(&doc).await?;
        info!("(Single crate) done: removed_count={}", removed_count);
        Ok(removed_count)
    }
}

/// Tests for our new `prune_invalid_category_slugs` logic.
/// Uses `#[traced_test]` for logging in test output.
#[cfg(test)]
mod test_prune_invalid_category_slugs {
    use super::*;

    /// A small helper to write a Cargo.toml with categories/keywords for testing.
    async fn write_cargo_toml_with_categories_and_keywords(dir: &Path, cat: &[&str], kw: &[&str]) {
        let cats = if cat.is_empty() {
            "[]".to_string()
        } else {
            let mut s = String::from("[");
            for (i, c) in cat.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push('"');
                s.push_str(c);
                s.push('"');
            }
            s.push(']');
            s
        };
        let kws = if kw.is_empty() {
            "[]".to_string()
        } else {
            let mut s = String::from("[");
            for (i, k) in kw.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push('"');
                s.push_str(k);
                s.push('"');
            }
            s.push(']');
            s
        };

        let cargo_toml_str = format!(
r#"[package]
name = "prune_example"
version = "0.1.0"
categories = {categories}
keywords   = {keywords}
"#,
            categories=cats,
            keywords=kws,
        );

        fs::write(dir.join("Cargo.toml"), cargo_toml_str)
            .await
            .expect("failed to write test Cargo.toml");
    }

    #[traced_test]
    async fn test_prune_categories_and_keywords_in_crate() {
        info!("Starting test_prune_categories_and_keywords_in_crate");
        let tmp = tempdir().expect("Failed to create temp dir");
        let root = tmp.path().to_path_buf();

        // Write a Cargo.toml with some categories and keywords
        let initial_cats = &["valid-cat", " has space", "in-val!d", "another_valid-cat"];
        let initial_keys = &["ok", "some space", "???", "also_ok"];
        write_cargo_toml_with_categories_and_keywords(&root, initial_cats, initial_keys).await;

        // Create the CrateHandle
        let mut handle: CrateHandle = CrateHandle::new(&root)
            .await
            .expect("Should create crate handle");

        let removed_count = handle.prune_invalid_category_slugs().await
            .expect("Failed to prune invalid slugs");
        // We expect " has space" to be removed, "in-val!d" to be removed from categories => 2 removed
        // We expect "some space" and "???" to be removed from keywords => 2 more removed
        // total 4 removed
        assert_eq!(removed_count, 4, "Expected to remove 4 total items");

        // Re-check the file to ensure they've been removed
        let after = fs::read_to_string(root.join("Cargo.toml")).await
            .expect("Failed to read updated Cargo.toml");
        info!("After prune:\n{}", after);

        // Confirm " has space" and "in-val!d" are gone from categories
        assert!(!after.contains(" has space"));
        assert!(!after.contains("in-val!d"));
        // Confirm "some space" and "???" are gone from keywords
        assert!(!after.contains("some space"));
        assert!(!after.contains("???"));

        // Confirm that "valid-cat" and "also_ok" remain
        assert!(after.contains("valid-cat"));
        assert!(after.contains("also_ok"));
    }
}
