// ---------------- [ File: workspacer-workspace/src/get_crates.rs ]
crate::ix!();

impl<P,H:CrateHandleInterface<P>> NumCrates for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    fn n_crates(&self) -> usize {
        self.crates().len()
    }
}

#[cfg(test)]
mod test_num_crates_and_get_crates {
    use super::*;

    #[traced_test]
    async fn test_empty_workspace_has_zero_crates() {
        // 1) Create an entirely empty workspace with no crates
        let path = create_mock_workspace(vec![]).await.expect("Failed to create empty workspace");
        // By default, create_mock_workspace writes a workspace Cargo.toml with no members if `crate_configs` is empty.
        
        // 2) Convert path -> a real Workspace
        let ws = Workspace::<PathBuf, CrateHandle>::new(&path).await
            .expect("Should build an empty workspace object");
        
        // 3) Check n_crates() and crates()
        assert_eq!(ws.n_crates(), 0, "No crates => n_crates==0");
        assert!(ws.crates().is_empty(), "crates() slice should be empty");
    }

    #[traced_test]
    async fn test_workspace_with_single_crate() {
        // 1) Create a mock workspace with 1 crate
        let path = create_mock_workspace(vec![
            CrateConfig::new("single_crate").with_src_files()
        ]).await.expect("Failed to create single-crate workspace");

        // 2) Build the workspace
        let ws = Workspace::<PathBuf,CrateHandle>::new(&path).await
            .expect("Should parse workspace with 1 crate");

        assert_eq!(ws.n_crates(), 1, "Expected exactly 1 crate");
        assert_eq!(ws.crates().len(), 1, "crates() slice should have length 1");
        // We can check the name or path if we want
        let the_crate = &ws.crates()[0];
        let crate_path = the_crate.as_ref();
        println!("Single crate path = {}", crate_path.display());
    }

    #[traced_test]
    async fn test_workspace_with_multiple_crates() {
        let path = create_mock_workspace(vec![
            CrateConfig::new("crateAlpha").with_src_files(),
            CrateConfig::new("crateBeta").with_test_files(),
        ]).await.expect("Failed to create multi-crate workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&path).await
            .expect("Should parse multi-crate workspace");

        assert_eq!(ws.n_crates(), 2, "Two crates => n_crates()==2");
        let slice = ws.crates();
        assert_eq!(slice.len(), 2, "crates() slice length=2");
    }
}
