// ---------------- [ File: src/handle_path_change.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #4: Handle an individual changed path
// ------------------------------------------------------------------------
pub async fn handle_path_change<P,H>(
    workspace: &Workspace<P,H>,
    path:      &Path,
    tx:        Option<&mpsc::Sender<Result<(), WorkspaceError>>>,
    runner:    &Arc<dyn CommandRunner + Send + Sync + 'static>,
) -> Result<(), WorkspaceError>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: WatchAndReload<Error=CrateError> + RebuildOrTest<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    if workspace.is_relevant_change(path) {
        info!("Detected relevant change in file: {:?}", path);

        let rebuild_result = workspace.rebuild_or_test(runner.as_ref()).await;
        notify_rebuild_result(tx, rebuild_result).await;
    }
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_handle_path_change {
    use super::*;
    #[tokio::test]
    async fn test_relevant_path_triggers_rebuild() {
        let workspace = mock_workspace_relevant_path();
        let runner = Arc::new(MockRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        // Suppose `workspace.is_relevant_change` returns true for "Cargo.toml"
        let path = Path::new("Cargo.toml");
        let result = handle_path_change(&workspace, path, Some(&tx), &runner).await;
        assert!(result.is_ok(), "No immediate error");

        // Check if a rebuild result was sent
        let msg = rx.recv().await;
        assert!(msg.is_some(), "Expected a rebuild result in the channel");
    }

    #[tokio::test]
    async fn test_irrelevant_path_no_rebuild() {
        let workspace = mock_workspace_irrelevant_path();
        let runner = Arc::new(MockRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        // Suppose workspace.is_relevant_change returns false
        let path = Path::new("random_file.txt");
        let result = handle_path_change(&workspace, path, Some(&tx), &runner).await;
        assert!(result.is_ok());

        // expect no message in channel
        let msg = rx.try_recv();
        assert!(msg.is_err(), "No message was sent if path is irrelevant");
    }
}
