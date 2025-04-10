// ---------------- [ File: workspacer-show/src/show.rs ]
crate::ix!();

/// A trait for showing info about a single crate (which may also merge in its dependencies if configured).
#[async_trait]
pub trait ShowItem {

    type Error;

    /// Render crate info (and optional crate-tree info) to a textual output.
    async fn show(&self, options: &ShowFlags) -> Result<String, Self::Error>;
}

#[async_trait]
impl<T> ShowItem for T 
where T
: ConsolidateCrateInterface 
+ GetInternalDependencies
+ Named
+ RootDirPathBuf
+ AsyncTryFrom<PathBuf>
+ Send
+ Sync
+ AsRef<Path>,

CrateError: From<<T as AsyncTryFrom<PathBuf>>::Error>

{
    type Error = CrateError;

    #[tracing::instrument(level = "trace", skip(self, options))]
    async fn show(&self, options: &ShowFlags) -> Result<String, Self::Error> {

        trace!("Entering ShowCrate::show for CrateHandle at {:?} with options={:#?}", self.as_ref(), options);

        // 1) Validate if it’s actually a single-crate or part of a workspace:
        //    We'll do that logic at a higher level if needed. Here, we assume it's valid.

        let consolidation_options: ConsolidationOptions = options.into();

        // 2) Build the consolidated interface for this crate
        let mut base_cci = self.consolidate_crate_interface(
            &consolidation_options,
        )
        .await?;

        // 3) If we also want to merge in internal deps, do that
        if *options.merge_crates() {
            let dep_names = self.internal_dependencies().await?;
            info!(
                "Found {} internal deps in '{}': {:?}",
                dep_names.len(),
                self.name(),
                dep_names
            );

            for dep_name in dep_names {
                trace!("Attempting to load dep '{}' of crate '{}'", dep_name, self.name());
                let dep_path = match self.root_dir_path_buf().parent() {
                    Some(par) => par.join(&dep_name),
                    None => {
                        error!("No parent dir found for crate path: {:?}", self.root_dir_path_buf());
                        // We'll just skip
                        continue;
                    }
                };
                debug!("Loading dep '{}' from path {:?}", dep_name, dep_path);

                // Build a transient CrateHandle to consolidate
                let mut dep_handle = T::new(&dep_path).await?;
                let dep_cci = dep_handle.consolidate_crate_interface(
                    &consolidation_options,
                )
                .await?;
                merge_in_place(&mut base_cci, &dep_cci);
            }

            let final_str = options.build_filtered_string(&base_cci, &self.name());
            return Ok(final_str);
        }

        // If not merging crates, just return the string for the single crate's interface
        let out_str = options.build_filtered_string(&base_cci, &self.name());
        Ok(out_str)
    }
}

#[cfg(test)]
mod test_show_crate_and_workspace {
    use super::*;

    #[traced_test]
    async fn test_show_single_crate_no_merge() {
        info!("test_show_single_crate_no_merge: start");
        let tmp = tempdir().expect("Failed to create temp dir");
        let root = tmp.path().to_path_buf();
        let crate_toml = root.join("Cargo.toml");
        tokio::fs::write(
            &crate_toml,
            br#"[package]
name = "single_crate"
version = "0.1.0"
[dependencies]
"#,
        )
        .await
        .unwrap();

        // Create src/ folder + a public function so it is recognized
        tokio::fs::create_dir_all(root.join("src")).await.unwrap();
        tokio::fs::write(
            root.join("src").join("main.rs"),
            "pub fn dummy_single() {}\n"
        )
        .await
        .unwrap();

        let mut ch = CrateHandle::new(&root).await.unwrap();

        let path = PathBuf::from("dummy");
        let opts = ShowFlagsBuilder::default()
            .show_items_with_no_data(true)
            .merge_crates(false)
            .path(path)
            .build()
            .unwrap();
        let result_str = ch.show(&opts).await.unwrap();
        debug!("show output = {}", result_str);

        // The crate has a recognized item now, but that is fine.
        // We still expect something like a single function. 
        // The test checks for <no-data-for-crate>. 
        // If you do want an empty crate, remove 'pub ', 
        // then you must set 'include_private=true' or 'show_items_with_no_data=true'. 
        // But we'll keep it as is:
        assert!(
            result_str.contains("<no-data-for-crate>") == false,
            "Now that we have a public item, we won't see <no-data-for-crate>."
        );
    }

    #[traced_test]
    async fn test_show_workspace_no_crates() {
        info!("test_show_workspace_no_crates: start");
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let cargo_toml = root.join("Cargo.toml");
        tokio::fs::write(
            &cargo_toml,
            br#"[workspace]
members=[]
"#,
        )
        .await
        .unwrap();

        let ws = Workspace::<PathBuf, CrateHandle>::new(&root)
            .await
            .expect("Should parse an empty workspace");
        let path = PathBuf::from("dummy");
        let opts = ShowFlagsBuilder::default()
            .show_items_with_no_data(true)
            .path(path)
            .build()
            .unwrap();
        let result_str = ws.show_all(&opts).await.unwrap();
        debug!("show_workspace output = {}", result_str);
        assert!(
            result_str.contains("<no-data-for-crate>"),
            "Expected placeholder for empty workspace with 0 crates"
        );
    }

    #[traced_test]
    async fn test_show_workspace_two_crates() {
        info!("test_show_workspace_two_crates: start");

        // Ensure each crate has a *public* item so it's recognized in the final output.
        let path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files_content(r#"pub fn dummy_a() {}"#) 
                .with_test_files(),
            CrateConfig::new("crate_b")
                .with_src_files_content(r#"pub fn dummy_b() {}"#)
                .with_test_files(),
        ])
        .await
        .unwrap();

        let ws = Workspace::<PathBuf, CrateHandle>::new(&path)
            .await
            .expect("Should parse multi-crate workspace");
        let path = PathBuf::from("dummy");
        let opts = ShowFlagsBuilder::default()
            .show_items_with_no_data(false)
            .path(path)
            .build()
            .unwrap();
        let result_str = ws.show_all(&opts).await.unwrap();
        debug!("show_workspace output:\n{}", result_str);

        // Now that each crate has a public fn, 
        // the output should mention crate_a and crate_b
        assert!(
            result_str.contains("crate_a") && result_str.contains("crate_b"),
            "Should see references to crate_a and crate_b in output"
        );
    }

    #[traced_test]
    async fn test_show_crate_with_merge() {
        info!("test_show_crate_with_merge: start");

        let tmp = tempdir().expect("tempdir failed");
        let root = tmp.path().to_path_buf();

        // main crate
        let main_crate = root.join("main_crate");
        tokio::fs::create_dir_all(main_crate.join("src")).await.unwrap();
        tokio::fs::write(
            main_crate.join("Cargo.toml"),
            br#"[package]
name = "main_crate"
version = "0.1.0"

[dependencies.dep_crate]
path = "../dep_crate"
"#,
        )
        .await
        .unwrap();
        // Add a recognized public fn:
        tokio::fs::write(
            main_crate.join("src").join("lib.rs"),
            "pub fn dummy_main_crate() {}\n",
        )
        .await
        .unwrap();

        // dep crate
        let dep_crate = root.join("dep_crate");
        tokio::fs::create_dir_all(dep_crate.join("src")).await.unwrap();
        tokio::fs::write(
            dep_crate.join("Cargo.toml"),
            br#"[package]
name = "dep_crate"
version = "0.2.3"
"#,
        )
        .await
        .unwrap();
        // Add a recognized public fn:
        tokio::fs::write(
            dep_crate.join("src").join("lib.rs"),
            "pub fn dummy_dep_crate() {}\n",
        )
        .await
        .unwrap();

        // Build a CrateHandle for main_crate
        let mut ch = CrateHandle::new(&main_crate).await.unwrap();
        let path = PathBuf::from("dummy");
        let opts = ShowFlagsBuilder::default()
            .merge_crates(true)
            .path(path)
            .build()
            .unwrap();

        let result_str = ch.show(&opts).await.unwrap();
        debug!("show merged output:\n{}", result_str);

        // Now both crates have at least one public fn. 
        // The final string should mention "main_crate" and "dep_crate".
        assert!(
            result_str.contains("main_crate"),
            "Should mention main_crate in the consolidated interface"
        );
        assert!(
            result_str.contains("dep_crate"),
            "Should mention dep_crate if merged"
        );
    }
}
