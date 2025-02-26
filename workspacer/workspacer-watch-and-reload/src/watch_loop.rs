// ---------------- [ File: src/watch_loop.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #2: The watch loop
// ------------------------------------------------------------------------
pub async fn watch_loop<P,H>(
    workspace:     &Workspace<P,H>,
    _watcher:      &mut RecommendedWatcher,
    _workspace_path: &PathBuf,
    notify_rx:     async_channel::Receiver<notify::Result<notify::Event>>,
    tx:            Option<mpsc::Sender<Result<(), WorkspaceError>>>,
    runner:        Arc<dyn CommandRunner + Send + Sync + 'static>,
    cancel_token:  CancellationToken,
) -> Result<(), WorkspaceError>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: WatchAndReload<Error=CrateError> + RebuildOrTest<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    loop {
        tokio::select! {
            // 2a) If we get a file event
            event_res = notify_rx.recv() => {
                match event_res {
                    Ok(res) => {
                        process_notify_event(workspace, res, tx.as_ref(), &runner).await?;
                    },
                    Err(_closed) => {
                        // channel closed
                        break;
                    }
                }
            },

            // 2b) If we get a cancellation
            _ = cancel_token.cancelled() => {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_watch_loop {
    use super::*;
    use workspacer_3p::tokio;
    use async_channel::bounded;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken; // or your usage
    use std::sync::Arc;
    use std::path::PathBuf;
    // Also define a mock runner

    #[tokio::test]
    async fn test_watch_loop_exits_on_channel_close() {
        // We'll create a mock notify channel
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();

        // Immediately drop the sender or close it => watch_loop sees 'Err(_closed)' => breaks
        drop(notify_tx);

        let workspace = mock_workspace();
        let runner = Arc::new(MockRunner::default());
        let cancel_token = CancellationToken::new();
        let mut watcher = create_dummy_watcher(); // We'll define a function or do an Option
        let path = PathBuf::from("/some/workspace");

        let result = watch_loop(
            &workspace,
            &mut watcher,
            &path,
            notify_rx,
            None,
            runner,
            cancel_token
        ).await;
        assert!(result.is_ok(), "If channel closed, watch_loop breaks with Ok(())");
    }

    #[tokio::test]
    async fn test_watch_loop_exits_on_cancel() {
        let (notify_tx, notify_rx) = async_channel::unbounded();
        let workspace = mock_workspace();
        let runner = Arc::new(MockRunner::default());
        let cancel_token = CancellationToken::new();
        let mut watcher = create_dummy_watcher();
        let path = PathBuf::from("/some/path");

        // spawn the watch_loop
        let join_handle = tokio::spawn(async move {
            watch_loop(
                &workspace,
                &mut watcher,
                &path,
                notify_rx,
                None,
                runner,
                cancel_token.clone()
            ).await
        });

        // request cancellation
        cancel_token.cancel();
        let result = join_handle.await.unwrap();
        assert!(result.is_ok(), "Should break on cancellation with Ok(())");
    }

    // You could also test feeding real or mock file events into notify_tx 
    // to see if the code calls `process_notify_event`.
}
