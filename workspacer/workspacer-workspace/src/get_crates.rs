// ---------------- [ File: src/get_crates.rs ]
crate::ix!();

impl<P,H:CrateHandleInterface<P>> NumCrates for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    fn n_crates(&self) -> usize {
        self.crates().len()
    }
}

#[cfg(test)]
#[disable]
mod test_num_crates_and_get_crates {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    // ----------------------------------------------------------------------
    // 1) Define a minimal mock or real `CrateHandleInterface` for H
    // ----------------------------------------------------------------------
    #[derive(Debug, Clone)]
    struct MockCrateHandle {
        path: PathBuf,
    }

    impl MockCrateHandle {
        fn new(path: PathBuf) -> Self {
            Self { path }
        }
    }

    // A trivial implementation to satisfy the trait bounds
    impl<P> CrateHandleInterface<P> for MockCrateHandle
    where
        // In real usage, you'd fill in details or rely on your existing code
        for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait,
    {
        // implement or stub out the required trait methods if needed
    }

    // ----------------------------------------------------------------------
    // 2) Define a minimal `Workspace<P, H>` mock that we can construct easily
    // ----------------------------------------------------------------------
    #[derive(Debug)]
    struct MockWorkspace<P,H> {
        path: P,
        crates: Vec<H>,
    }

    impl<P,H> MockWorkspace<P,H> {
        fn new(path: P, crates: Vec<H>) -> Self {
            Self { path, crates }
        }
    }

    // If `MockWorkspace` needs to implement your `WorkspaceInterface<P,H>` or `NumCrates`/`GetCrates`,
    // we can do so. For demonstration, let's do it directly:
    impl<P,H: CrateHandleInterface<P>> NumCrates for MockWorkspace<P,H> 
    where 
        for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
    {
        fn n_crates(&self) -> usize {
            self.crates.len()
        }
    }

    impl<P,H: CrateHandleInterface<P>> GetCrates<P,H> for MockWorkspace<P,H> 
    where
        for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
    {
        fn crates(&self) -> &[H] {
            &self.crates
        }
    }

    // ----------------------------------------------------------------------
    // 3) Tests for n_crates() and crates()
    // ----------------------------------------------------------------------

    /// Scenario: Empty workspace (no crates).
    #[test]
    fn test_empty_workspace() {
        let path = PathBuf::from("/fake/workspace");
        let crates = vec![];  // no crates
        let ws = MockWorkspace::new(path, crates);

        assert_eq!(ws.n_crates(), 0, "No crates => n_crates should be 0");
        assert!(ws.crates().is_empty(), "crates() slice should be empty");
    }

    /// Scenario: Workspace with a single crate.
    #[test]
    fn test_single_crate_workspace() {
        let path = PathBuf::from("/workspace/single");
        let crate_handle = MockCrateHandle::new(PathBuf::from("/workspace/single/crateA"));
        let ws = MockWorkspace::new(path, vec![crate_handle.clone()]);

        assert_eq!(ws.n_crates(), 1, "One crate => n_crates should be 1");
        let slice = ws.crates();
        assert_eq!(slice.len(), 1, "crates() slice should have length 1");
        // Check that the data matches
        assert_eq!(slice[0].path, crate_handle.path);
    }

    /// Scenario: Workspace with multiple crates.
    #[test]
    fn test_multiple_crates_workspace() {
        let path = PathBuf::from("/workspace/multi");
        let crateA = MockCrateHandle::new(PathBuf::from("/workspace/multi/crateA"));
        let crateB = MockCrateHandle::new(PathBuf::from("/workspace/multi/crateB"));
        let ws = MockWorkspace::new(path, vec![crateA.clone(), crateB.clone()]);

        assert_eq!(ws.n_crates(), 2, "Two crates => n_crates should be 2");
        let slice = ws.crates();
        assert_eq!(slice.len(), 2, "crates() slice should have length 2");

        // Check that the items match the ones we inserted
        assert_eq!(slice[0].path, crateA.path);
        assert_eq!(slice[1].path, crateB.path);
    }

    /// Scenario: n_crates and crates remain consistent if we add or remove crates
    /// (if your code allows mutation). Demonstrates that the slice points to the same data.
    #[test]
    fn test_consistency_after_modification() {
        let path = PathBuf::from("/workspace/mutable");
        let mut crates = vec![
            MockCrateHandle::new(PathBuf::from("/workspace/mutable/crate1")),
        ];
        let mut ws = MockWorkspace::new(path, crates);

        // Initially 1 crate
        assert_eq!(ws.n_crates(), 1);
        assert_eq!(ws.crates().len(), 1);

        // Simulate adding another crate
        // In real code, you might have a method for that, or you might just mutate ws.crates
        ws.crates.push(MockCrateHandle::new(PathBuf::from("/workspace/mutable/crate2")));
        assert_eq!(ws.n_crates(), 2);
        assert_eq!(ws.crates().len(), 2);

        // Remove one crate
        ws.crates.pop();
        assert_eq!(ws.n_crates(), 1);
        assert_eq!(ws.crates().len(), 1);
    }
}
