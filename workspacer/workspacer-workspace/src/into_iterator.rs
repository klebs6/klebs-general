// ---------------- [ File: workspacer-workspace/src/into_iterator.rs ]
crate::ix!();

impl<'a,P,H:CrateHandleInterface<P>> IntoIterator for &'a Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Item     = &'a Arc<AsyncMutex<H>>;
    type IntoIter = Iter<'a, Arc<AsyncMutex<H>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.crates().iter()
    }
}

#[cfg(test)]
mod test_into_iterator {
    use super::*;

    #[traced_test]
    async fn test_real_workspace_into_iterator() {
        // 1) Create a mock workspace on disk with 2 crates
        let path = create_mock_workspace(vec![
            CrateConfig::new("crateA").with_src_files(),
            CrateConfig::new("crateB").with_src_files(),
        ]).await.expect("Failed to create mock workspace");

        // 2) Convert path -> Workspace<PathBuf, CrateHandle>
        let ws = Workspace::<PathBuf, CrateHandle>::new(&path)
            .await
            .expect("Should create workspace from path");

        // 3) Now test the into_iterator for &ws
        let crates_vec: Vec<_> = ws.into_iter().collect();
        assert_eq!(crates_vec.len(), 2, "Should have 2 crates in iteration");
    }
}
