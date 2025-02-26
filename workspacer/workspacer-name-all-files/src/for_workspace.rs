// ---------------- [ File: src/for_workspace.rs ]
crate::ix!();

// Implementation for an entire workspace. Iterates over all crates, calling `name_all_files`
// on each one. Aggregates any crate-level failures into `WorkspaceError::MultipleErrors`.
#[async_trait]
impl<P,H> NameAllFiles for Workspace<P,H>
where
    // your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: NameAllFiles<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    type Error = WorkspaceError;


    async fn name_all_files(&self) -> Result<(), Self::Error> {
        let mut errors = Vec::new();

        for crate_handle in self {
            if let Err(e) = crate_handle.name_all_files().await {
                // Wrap the `CrateError` in a `WorkspaceError` variant:
                errors.push(WorkspaceError::CrateError(e.into()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}

#[cfg(test)]
#[disable]
mod test_workspace_name_all_files_for_workspace {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use workspacer_3p::tokio;

    // If you already have a `Workspace<P,H>` type that can be constructed from
    // a path with multiple crates, you can do something like:
    //   let root_path = create_mock_workspace(vec![...]) // your mock method
    //   let ws = Workspace::new(&root_path).await?;
    //
    // Then you call `ws.name_all_files().await`.

    // We'll show a partial example:

    #[tokio::test]
    async fn test_name_all_files_all_crates_succeed() {
        // 1) Create a workspace with multiple crates
        // For demonstration, assume you have a helper that does so:
        //   let crate_configs = vec![CrateConfig::new("crate_a").with_src_files(), CrateConfig::new("crate_b").with_src_files()];
        //   let root_path = create_mock_workspace(crate_configs).await.expect("workspace creation");
        //   let ws = Workspace::new(&root_path).await.expect("create workspace");
        //
        // For brevity, we’ll omit the full creation code and just illustrate:

        // Suppose we already have the workspace:
        let ws = make_test_workspace_with_two_crates().await; // hypothetical helper

        // 2) Call name_all_files
        let result = ws.name_all_files().await;

        // 3) If both crates' name_all_files succeeded, we get Ok(())
        assert!(result.is_ok(), "Expected success if all crates succeeded");

        // Optionally verify that each .rs file in each crate was named or tagged
        // e.g., by checking for lines like `// ------ [ File: ... ]` in the .rs files
        // if that's what `name_all_files()` does.
        // ...
    }

    #[tokio::test]
    async fn test_name_all_files_one_crate_fails() {
        // Create a workspace with two crates:
        //   crate_a -> name_all_files works
        //   crate_b -> name_all_files fails
        // We'll simulate or forcibly cause crate_b's name_all_files to fail
        let ws = make_test_workspace_with_failing_crate().await;

        let result = ws.name_all_files().await;
        match result {
            Err(WorkspaceError::MultipleErrors(errors)) => {
                // We expect exactly one error from crate_b
                assert_eq!(errors.len(), 1, "Should have exactly one crate error");
                match &errors[0] {
                    WorkspaceError::CrateError(e) => {
                        // Possibly check e
                        println!("CrateError: {:?}", e);
                    }
                    other => panic!("Expected a CrateError variant, got {:?}", other),
                }
            }
            Ok(_) => {
                panic!("Expected one failing crate => should return MultipleErrors");
            }
            other => panic!("Expected MultipleErrors, got {:?}", other),
        }
    }
    // For completeness, your `name_all_files()` in each crate might rename or tag .rs files.
    // If you do real file manipulation, the test checks for actual changes on disk. 
}
