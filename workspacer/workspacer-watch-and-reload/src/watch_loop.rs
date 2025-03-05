// ---------------- [ File: workspacer-watch-and-reload/src/watch_loop.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #2: The watch loop
// ------------------------------------------------------------------------
pub async fn watch_loop<X,E>(
    watched:       &X,
    _watcher:      &mut RecommendedWatcher,
    _workspace_path: &PathBuf,
    notify_rx:     async_channel::Receiver<notify::Result<notify::Event>>,
    tx:            Option<mpsc::Sender<Result<(), E>>>,
    runner:        Arc<dyn CommandRunner + Send + Sync + 'static>,
    cancel_token:  CancellationToken,
) -> Result<(), E>
where
    X: WatchAndReload<Error=E> + RebuildOrTest<Error=E>,
    E: From<WatchError>,
{
    loop {
        let cancel_clone = cancel_token.clone();
        tokio::select! {
            // 2a) If we get a file event
            event_res = notify_rx.recv() => {
                match event_res {
                    Ok(res) => {
                        process_notify_event(watched, res, tx.as_ref(), &runner).await?;
                    },
                    Err(_closed) => {
                        // channel closed
                        break;
                    }
                }
            },

            // 2b) If we get a cancellation
            _ = cancel_clone.cancelled() => {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_watch_loop {
    use super::*;

    // We'll define a local "create_dummy_watcher"
    fn create_dummy_watcher() -> RecommendedWatcher {
        use notify::Config;
        RecommendedWatcher::new(
            |_res| {},
            Config::default(),
        ).expect("dummy watcher init")
    }

    #[derive(Default)]
    struct MockCommandRunner;
    impl CommandRunner for MockCommandRunner {
        fn run_command(&self, _cmd: tokio::process::Command)
            -> tokio::task::JoinHandle<Result<std::process::Output, std::io::Error>>
        {
            tokio::spawn(async {
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"mock build/test success".to_vec(),
                    stderr: vec![],
                })
            })
        }
    }

    #[traced_test]
    async fn test_watch_loop_exits_on_channel_close() {
        info!("Starting test_watch_loop_exits_on_channel_close");
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();
        drop(notify_tx); // channel closed immediately

        let workspace_path = create_mock_workspace(vec![]).await.unwrap();
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&workspace_path).await.unwrap();
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();
        let mut watcher = create_dummy_watcher();
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

    #[traced_test]
    async fn test_watch_loop_exits_on_cancel() {
        info!("Starting test_watch_loop_exits_on_cancel");
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();
        let workspace_path = create_mock_workspace(vec![]).await.unwrap();
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&workspace_path).await.unwrap();
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();
        let mut watcher = create_dummy_watcher();
        let path = PathBuf::from("/some/path");

        let join_handle = tokio::spawn(async move {
            watch_loop(
                &workspace,
                &mut watcher,
                &path,
                notify_rx,
                None,
                runner,
                cancel_clone
            ).await
        });

        cancel_token.cancel();
        let result = join_handle.await.unwrap();
        assert!(result.is_ok(), "Should break on cancellation with Ok(())");
    }
}
