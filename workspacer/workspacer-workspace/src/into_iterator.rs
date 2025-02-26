// ---------------- [ File: src/into_iterator.rs ]
crate::ix!();

impl<'a,P,H:CrateHandleInterface<P>> IntoIterator for &'a Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Item     = &'a H;
    type IntoIter = Iter<'a, H>;

    fn into_iter(self) -> Self::IntoIter {
        self.crates().iter()
    }
}

// =============================================
// Test Module #1: IntoIterator for &Workspace
// =============================================
#[cfg(test)]
#[disable]
mod test_into_iterator {
    use super::*;
    #[test]
    fn test_empty_workspace_iter() {
        let ws = MockWorkspace::new(
            MockPath(PathBuf::from("/some/where")),
            vec![] as Vec<MockCrateHandle>
        );
        let mut iter = ws.into_iter();
        assert_eq!(iter.next(), None, "No crates => iteration is empty");
    }

    #[test]
    fn test_single_crate_iter() {
        let c1 = MockCrateHandle { crate_path: PathBuf::from("crateA"), publish_ready: true };
        let ws = MockWorkspace::new(MockPath(PathBuf::from("/single")), vec![c1]);

        let mut iter = ws.into_iter();
        let first = iter.next();
        assert!(first.is_some(), "Should have 1 crate");
        assert_eq!(first.unwrap().crate_path, PathBuf::from("crateA"));
        assert_eq!(iter.next(), None, "No more crates after the first");
    }

    #[test]
    fn test_multiple_crates_iter() {
        let c1 = MockCrateHandle { crate_path: PathBuf::from("crateA"), publish_ready: true };
        let c2 = MockCrateHandle { crate_path: PathBuf::from("crateB"), publish_ready: true };
        let ws = MockWorkspace::new(MockPath(PathBuf::from("/multi")), vec![c1.clone(), c2.clone()]);

        let crates_vec: Vec<_> = ws.into_iter().collect();
        assert_eq!(crates_vec.len(), 2, "Should iterate over 2 crates");
        assert_eq!(crates_vec[0].crate_path, c1.crate_path);
        assert_eq!(crates_vec[1].crate_path, c2.crate_path);
    }
}
