// ---------------- [ File: workspacer/src/watch_and_reload.rs ]
crate::ix!();

#[async_trait]
impl<P,H:CrateHandleInterface<P>> WatchAndReload for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    async fn watch_and_reload(
        &self,
        tx:           Option<mpsc::Sender<Result<(), WorkspaceError>>>,
        runner:       Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,

    ) -> Result<(), WorkspaceError> {

        let workspace_path = self.as_ref().to_path_buf();

        // Channel for receiving file change events
        let (notify_tx, notify_rx) = async_channel::unbounded();

        // Create a `notify` file watcher
        let notify_tx_clone = notify_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                // Send the event over the async_channel
                let _ = notify_tx_clone.try_send(res);
            },
            notify::Config::default(),
        )
        .map_err(|e| WatchError::NotifyError(e.into()))?;

        // Watch the workspace directory recursively
        watcher
            .watch(&workspace_path, RecursiveMode::Recursive)
            .map_err(|e| WatchError::NotifyError(e.into()))?;

        // Keep the watcher alive
        let _watcher = watcher;

        // Process events from the async_channel
        loop {
            tokio::select! {
                res = notify_rx.recv() => {
                    match res {
                        Ok(res) => match res {
                            Ok(event) => {
                                for path in event.paths.iter() {
                                    if self.is_relevant_change(&path) {
                                        info!("Detected change in file: {:?}", path);

                                        // Trigger rebuild or tests on file change
                                        let rebuild_result = self.rebuild_or_test(runner.as_ref()).await;

                                        if let Some(ref sender) = tx {
                                            let _ = sender.send(rebuild_result).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("File watch error: {:?}", e);
                                let e: Arc<notify::Error> = Arc::new(e);
                                if let Some(ref sender) = tx {
                                    let _ = sender.send(Err(WorkspaceError::from(e.clone()))).await;
                                }
                                return Err(WorkspaceError::from(e));
                            }
                        },
                        Err(_) => {
                            // The channel has been closed
                            break;
                        }
                    }
                },
                _ = cancel_token.cancelled() => {
                    // Received cancellation signal
                    break;
                }
            }
        }

        Ok(())
    }

    /// Determines if a file change is relevant (e.g., in `src/` or `Cargo.toml` files).
    fn is_relevant_change(&self, path: &Path) -> bool {
        // Check if the path ends with 'Cargo.toml'
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            return true;
        }

        // Check if the path is within any crate's 'src/' directory
        for crate_handle in self.crates() {
            let crate_src_path = crate_handle.as_ref().join("src");
            if path.starts_with(&crate_src_path) {
                return true;
            }
        }

        false
    }
}
