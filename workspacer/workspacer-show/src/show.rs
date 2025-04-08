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
        trace!("Entering ShowCrate::show_crate for CrateHandle at {:?}", self.as_ref());

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

// --------------------------- TESTS FOR THIS CRATE ---------------------------
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

        // Build CrateHandle
        let mut ch = CrateHandle::new(&root).await.unwrap();

        let opts = ShowFlagsBuilder::new()
            .show_items_with_no_data(true)
            .merge_crates(false)
            .build()
            .unwrap();
        let result_str = ch.show_crate(&opts).await.unwrap();
        debug!("show_crate output = {}", result_str);

        // The consolidated interface is likely empty, so we expect <no-data-for-crate>
        assert!(
            result_str.contains("<no-data-for-crate>"),
            "Expected placeholder for empty crate"
        );
    }

    #[traced_test]
    async fn test_show_workspace_no_crates() {
        info!("test_show_workspace_no_crates: start");
        // We'll create a minimal Cargo.toml that is indeed a workspace, but with zero members.
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
        let opts = ShowFlagsBuilder::new()
            .show_items_with_no_data(true)
            .build()
            .unwrap();
        let result_str = ws.show_workspace(&opts).await.unwrap();
        debug!("show_workspace output = {}", result_str);
        assert!(
            result_str.contains("<no-data-for-crate>"),
            "Expected placeholder for empty workspace with 0 crates"
        );
    }

    #[traced_test]
    async fn test_show_workspace_two_crates() {
        info!("test_show_workspace_two_crates: start");

        let path = create_mock_workspace(vec![
            CrateConfig::new("crate_a").with_src_files(),
            CrateConfig::new("crate_b").with_test_files(),
        ])
        .await
        .unwrap();

        let ws = Workspace::<PathBuf, CrateHandle>::new(&path)
            .await
            .expect("Should parse multi-crate workspace");
        let opts = ShowFlagsBuilder::new()
            .show_items_with_no_data(false)
            .build()
            .unwrap();
        let result_str = ws.show_workspace(&opts).await.unwrap();
        debug!("show_workspace output:\n{}", result_str);
        // We expect to see something about crate_a and crate_b (though actual contents might be minimal)
        assert!(result_str.contains("crate_a") && result_str.contains("crate_b"),
            "Should see references to crate_a and crate_b in output"
        );
    }

    #[traced_test]
    async fn test_show_crate_with_merge() {
        info!("test_show_crate_with_merge: start");
        // We'll create a single "main" crate, plus an internal dep. Because it's a single crate scenario
        // that references a local path. Then we set merge_crates=true and ensure it merges the dep's interface.

        let tmp = tempdir().expect("tempdir failed");
        let root = tmp.path().to_path_buf();

        // 1) main crate
        let main_crate = root.join("main_crate");
        tokio::fs::create_dir_all(&main_crate).await.unwrap();
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

        // 2) dep crate
        let dep_crate = root.join("dep_crate");
        tokio::fs::create_dir_all(&dep_crate).await.unwrap();
        tokio::fs::write(
            dep_crate.join("Cargo.toml"),
            br#"[package]
name = "dep_crate"
version = "0.2.3"
"#,
        )
        .await
        .unwrap();

        // 3) Build a CrateHandle for main_crate
        let mut ch = CrateHandle::new(&main_crate).await.unwrap();
        let opts = ShowFlagsBuilder::new()
            .merge_crates(true)
            .build()
            .unwrap();

        let result_str = ch.show_crate(&opts).await.unwrap();
        debug!("show_crate merged output:\n{}", result_str);
        // We expect to see references to "main_crate" and "dep_crate"
        // because we merged them
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
