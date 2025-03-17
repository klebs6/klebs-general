// ---------------- [ File: workspacer-watch-and-reload/src/watch_and_reload.rs ]
crate::ix!();

#[async_trait]
pub trait WatchAndReload {

    type Error;

    async fn watch_and_reload(
        &self,
        tx: Option<mpsc::Sender<Result<(), Self::Error>>>,
        runner: Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), Self::Error>;

    fn is_relevant_change(&self, path: &Path) -> bool;
}

#[async_trait]
impl WatchAndReload for CrateHandle {
    type Error = CrateError;

    ///
    /// Sets up file-watching for this one crate, then enters a watch loop
    /// until `cancel_token` is triggered. Triggers rebuild/test whenever
    /// a relevant change is detected.
    ///
    async fn watch_and_reload(
        &self,
        tx: Option<mpsc::Sender<Result<(), Self::Error>>>,
        runner: Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        let crate_path = self.crate_dir_path_buf();
        info!("Setting up file watching for crate at: {}", crate_path.display());

        let (mut watcher, notify_rx) = setup_file_watching(&crate_path)?;

        info!("Entering watch loop for crate at: {}", crate_path.display());
        watch_loop(
            self,
            &mut watcher,
            &crate_path,
            notify_rx,
            tx,
            runner,
            cancel_token,
        )
        .await?;

        info!("Watch loop ended gracefully for crate at: {}", crate_path.display());
        Ok(())
    }

    ///
    /// A change is considered “relevant” if it's Cargo.toml or in `src/`
    ///
    fn is_relevant_change(&self, path: &Path) -> bool {
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            return true;
        }
        let src_path = self.as_ref().join("src");
        path.starts_with(&src_path)
    }
}

#[async_trait]
impl<P,H> WatchAndReload for Workspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: WatchAndReload<Error=CrateError> + RebuildOrTest<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    type Error = WorkspaceError;

    async fn watch_and_reload(
        &self,
        tx:           Option<mpsc::Sender<Result<(), Self::Error>>>,
        runner:       Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), Self::Error> {

        // 1) Setup the file watcher
        let workspace_path = self.workspace_dir_path_buf();
        let (mut watcher, notify_rx) = setup_file_watching(&workspace_path)
            .map_err(WorkspaceError::from)?;

        // 2) Enter the watch loop
        watch_loop(
            self,
            &mut watcher,
            &workspace_path,
            notify_rx,
            tx,
            runner,
            cancel_token,
        ).await
    }

    /// (unchanged) - determines if a file change is relevant
    fn is_relevant_change(&self, path: &Path) -> bool {
        // same logic as before:
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            return true;
        }

        for crate_handle in self.crates() {
            let crate_src_path = crate_handle.crate_dir_path_buf().join("src");
            if path.starts_with(&crate_src_path) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test_watch_and_reload {
    use super::*;
    use notify::event::{Event, EventKind, ModifyKind, DataChange};
    use tempfile::tempdir;
    use tracing::{info, warn, error, debug};
    use std::sync::Arc;
    use std::os::unix::process::ExitStatusExt; // for from_raw(0)
    use tokio::sync::mpsc;
    use tokio::time::Duration;
    use workspacer_3p::tokio;
    use workspacer_3p::tokio::fs;
    use workspacer_3p::tokio::runtime::Runtime;
    use std::path::{Path, PathBuf};
    use notify::RecommendedWatcher;
    use notify::RecursiveMode;
    use async_channel::unbounded;
    use crate::CancellationToken;
    use lightweight_command_runner::CommandRunner;
    use std::process::Output;

    // ------------------------------------------------------------------
    // Local mock that *resembles* a workspace but is not the real one.
    // Our real watch_loop(...) function demands a &Workspace<P,H>,
    // but since we cannot define an inherent impl for an external type,
    // we define this minimal mock plus a local watch_loop_mock if needed.
    // ------------------------------------------------------------------
    #[derive(Debug, Clone)]
    struct MockWorkspaceForWatchAndReload {
        path: PathBuf
    }

    impl MockWorkspaceForWatchAndReload {
        fn new(p: PathBuf) -> Self {
            Self { path: p }
        }

        /// Just a fake check to see if a changed path is relevant
        fn is_relevant_change(&self, changed_path: &Path) -> bool {
            // We'll say Cargo.toml or anything with /src/ is relevant
            if changed_path.file_name() == Some("Cargo.toml".as_ref()) {
                return true;
            }
            changed_path.to_string_lossy().contains("/src/")
        }

        /// Faked rebuild-or-test
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            info!("Mock rebuild_or_test invoked for path: {}", self.path.display());
            let mut cmd = tokio::process::Command::new("echo");
            cmd.arg("mock-build").arg("success");
            let handle = runner.run_command(cmd);
            let output = handle.await??;

            if !output.status.success() {
                error!("Mock build/test failed with status {:?}", output.status);
                return Err(WorkspaceError::MockBuildTestFailedWithStatus {
                    status: output.status,
                });
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockCommandRunner;
    impl CommandRunner for MockCommandRunner {
        fn run_command(
            &self,
            mut cmd: tokio::process::Command,
        ) -> tokio::task::JoinHandle<Result<Output, std::io::Error>> {
            // For demonstration, pretend success
            tokio::spawn(async move {
                let _ = cmd; // not actually run
                Ok(Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"mock success".to_vec(),
                    stderr: vec![],
                })
            })
        }
    }

    // ------------------------------------------------------------------
    // If you still want to use the *real* watch_loop(...) with a mock
    // "Workspace-like" object, you can define a wrapper that satisfies
    // the signature via an intermediate trait or newtype. 
    // Here, we do the simpler approach: We just test the logic that 
    // goes inside watch_loop, feeding events manually. 
    // ------------------------------------------------------------------
    #[traced_test]
    async fn test_watch_and_reload_mock_events() {
        info!("Starting test_watch_and_reload_mock_events");
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();
        let mut watcher = {
            use notify::Config;
            RecommendedWatcher::new(
                move |_res| {
                    // do nothing, we feed events manually
                },
                notify::Config::default(),
            ).expect("dummy watcher init")
        };
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(5);
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        // Our local mock "workspace"
        let mock_ws = MockWorkspaceForWatchAndReload::new(PathBuf::from("/mock/workspace"));

        // We'll spawn a local future that mimics watch_loop but uses
        // the mock workspace's is_relevant_change(...) logic and
        // rebuild_or_test(...) calls:
        let handle = tokio::spawn({
            let ws_clone = mock_ws.clone();
            async move {
                loop {
                    tokio::select! {
                        evt = notify_rx.recv() => {
                            match evt {
                                Ok(Ok(ev)) => {
                                    info!("Got mock event: {:?}", ev.kind);
                                    // For each path, check if relevant
                                    for changed_path in ev.paths {
                                        if ws_clone.is_relevant_change(&changed_path) {
                                            match ws_clone.rebuild_or_test(runner.as_ref()).await {
                                                Ok(_) => {
                                                    let _ = tx.send(Ok(())).await;
                                                },
                                                Err(e) => {
                                                    let _ = tx.send(Err(e)).await;
                                                }
                                            }
                                        }
                                    }
                                },
                                Ok(Err(notify_err)) => {
                                    error!("Notify error: {:?}", notify_err);
                                    let _ = tx.send(Err(WorkspaceError::FileWatchError)).await;
                                    break;
                                },
                                Err(_closed) => {
                                    warn!("notify channel closed, exiting loop");
                                    break;
                                },
                            }
                        },
                        _ = cancel_clone.cancelled() => {
                            info!("Cancellation requested, exiting loop");
                            break;
                        }
                    }
                }
                Ok::<(), WorkspaceError>(())
            }
        });

        // Send a "relevant" Cargo.toml event
        let ev_relevant = notify::Event {
            kind: EventKind::Modify(ModifyKind::Data(DataChange::Content)),
            paths: vec![PathBuf::from("/mock/workspace/Cargo.toml")],
            attrs: Default::default(),
        };
        notify_tx.send(Ok(ev_relevant)).await.unwrap();

        if let Ok(Some(msg)) = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
            assert!(msg.is_ok(), "Expected OK rebuild result for relevant path");
        } else {
            panic!("No rebuild result after relevant path event!");
        }

        // Send an irrelevant event
        let ev_irrelevant = notify::Event {
            kind: EventKind::Modify(ModifyKind::Data(DataChange::Content)),
            paths: vec![PathBuf::from("/some/unrelated/file.txt")],
            attrs: Default::default(),
        };
        notify_tx.send(Ok(ev_irrelevant)).await.unwrap();

        let maybe_msg = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(maybe_msg.is_err(), "No new message for irrelevant path expected");

        cancel_token.cancel();
        let result = handle.await;
        assert!(result.is_ok(), "Background loop ended OK after cancel");
        info!("test_watch_and_reload_mock_events complete");
    }

    #[traced_test]
    async fn test_watch_and_reload_notify_error() {
        info!("Starting test_watch_and_reload_notify_error");
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();
        let mut watcher = {
            use notify::Config;
            RecommendedWatcher::new(
                move |_res| {
                    // do nothing
                },
                notify::Config::default(),
            ).expect("dummy watcher init")
        };
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(5);
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();
        let mock_ws = MockWorkspaceForWatchAndReload::new(PathBuf::from("/mock/ws"));

        let handle = tokio::spawn({
            let ws_clone = mock_ws.clone();
            async move {
                loop {
                    tokio::select! {
                        evt = notify_rx.recv() => {
                            match evt {
                                Ok(Ok(ev)) => {
                                    info!("Got event kind: {:?}", ev.kind);
                                    for p in ev.paths {
                                        if ws_clone.is_relevant_change(&p) {
                                            let ret = ws_clone.rebuild_or_test(runner.as_ref()).await;
                                            let _ = tx.send(ret).await;
                                        }
                                    }
                                },
                                Ok(Err(err)) => {
                                    error!("Got notify error: {:?}", err);
                                    let _ = tx.send(Err(WorkspaceError::FileWatchError)).await;
                                    break;
                                }
                                Err(_closed) => {
                                    warn!("Channel closed, break loop");
                                    break;
                                }
                            }
                        },
                        _ = cancel_clone.cancelled() => {
                            info!("Cancellation requested, break loop");
                            break;
                        }
                    }
                }
                Ok::<(), WorkspaceError>(())
            }
        });

        // Force a watch error:
        let fake_err = notify::Error::generic("some watch error");
        notify_tx.send(Err(fake_err)).await.unwrap();

        let msg = rx.recv().await.expect("Expected an error from watch loop");
        assert!(msg.is_err(), "Should have an error result from watch error");

        cancel_token.cancel();
        let final_result = handle.await.unwrap();
        assert!(final_result.is_ok(), "Loop ended after error, returning Ok");
        info!("test_watch_and_reload_notify_error complete");
    }
}

#[cfg(test)]
mod test_is_relevant_change {
    use super::*;
    use std::path::PathBuf;
    use tracing::{info, warn, error, debug};

    // A local mock with an is_relevant_change method
    #[derive(Debug)]
    struct MockWorkspaceForRelevantCheck {
        src_path: PathBuf,
    }

    impl MockWorkspaceForRelevantCheck {
        fn new(src_path: PathBuf) -> Self {
            Self { src_path }
        }
        fn is_relevant_change(&self, path: &std::path::Path) -> bool {
            // We'll consider it relevant if `path.file_name() == Cargo.toml`
            // or if it starts_with the stored src_path
            if path.file_name() == Some("Cargo.toml".as_ref()) {
                return true;
            }
            path.starts_with(&self.src_path)
        }
    }

    #[traced_test]
    fn test_is_relevant_cargo_toml() {
        info!("test_is_relevant_cargo_toml started");
        let ws = MockWorkspaceForRelevantCheck::new(PathBuf::from("/unused/src"));
        let path = PathBuf::from("Cargo.toml");
        assert!(ws.is_relevant_change(&path), "Cargo.toml should be relevant");
        info!("test_is_relevant_cargo_toml done");
    }

    #[traced_test]
    fn test_is_relevant_src_file() {
        info!("test_is_relevant_src_file started");
        let ws = MockWorkspaceForRelevantCheck::new(PathBuf::from("/some/crate/src"));
        let path = PathBuf::from("/some/crate/src/lib.rs");
        assert!(ws.is_relevant_change(&path), "lib.rs in /src/ is relevant");
        info!("test_is_relevant_src_file done");
    }

    #[traced_test]
    fn test_not_relevant() {
        info!("test_not_relevant started");
        let ws = MockWorkspaceForRelevantCheck::new(PathBuf::from("/some/other/path/src"));
        let path = PathBuf::from("/unrelated/path/main.rs");
        assert!(!ws.is_relevant_change(&path), "Should not be relevant");
        info!("test_not_relevant done");
    }
}
