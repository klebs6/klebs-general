// ---------------- [ File: src/watch_and_reload.rs ]
crate::ix!();

#[async_trait]
pub trait WatchAndReload {

    type Error;

    async fn watch_and_reload(
        &self,
        tx: Option<mpsc::Sender<Result<(), WorkspaceError>>>,
        runner: Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), Self::Error>;

    fn is_relevant_change(&self, path: &Path) -> bool;
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
        tx:           Option<mpsc::Sender<Result<(), WorkspaceError>>>,
        runner:       Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), WorkspaceError> {

        // 1) Setup the file watcher
        let workspace_path = self.as_ref().to_path_buf();
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
            let crate_src_path = crate_handle.as_ref().join("src");
            if path.starts_with(&crate_src_path) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
#[disable]
mod test_watch_and_reload {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use workspacer_3p::tokio::runtime::Runtime;
    use workspacer_3p::tokio;
    use crate::CommandRunner; // or your actual references
    use crate::{WatchAndReload, setup_file_watching, watch_loop, WorkspaceError, WatchError};
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode};
    use async_channel;

    // -----------------------------------------------------------------------
    // 1) Real Approach: Actually watch a real filesystem directory
    // -----------------------------------------------------------------------
    // This is more of an integration test. We create a temp directory,
    // spawn watch_and_reload, then physically write to a file,
    // and see if we get a relevant event.

    use tempfile::tempdir;
    use tokio_util::sync::CancellationToken;

    // -------------------------------------------------------------------
    // Mocks used in the above tests
    // -------------------------------------------------------------------
    #[derive(Clone, Debug)]
    struct MockWorkspace {
        path: PathBuf,
    }
    impl MockWorkspace {
        fn new(path: PathBuf) -> Self {
            Self { path }
        }
    }
    impl AsRef<Path> for MockWorkspace {
        fn as_ref(&self) -> &Path { &self.path }
    }
    impl<P,H> WorkspaceInterface<P,H> for MockWorkspace { /* stubs, or not used in test? */ }
    impl<P,H> WatchAndReload for MockWorkspace { /* if needed, or we skip if we directly call watch_loop */ }

    // If you want a real crate trait implementation, define them. 
    // We'll define is_relevant_change => Cargo.toml or .src:
    impl MockWorkspace {
        fn is_relevant_change(&self, path: &Path) -> bool {
            path.file_name() == Some("Cargo.toml".as_ref())
            || path.to_string_lossy().contains("/src/")
        }
        async fn rebuild_or_test(&self, _runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            // returns Ok
            Ok(())
        }
    }

    // A mock runner
    #[derive(Default)]
    struct MockCommandRunner;
    impl CommandRunner for MockCommandRunner {
        fn run_command(&self, mut cmd: tokio::process::Command) 
            -> tokio::task::JoinHandle<Result<std::process::Output, std::io::Error>> 
        {
            // For demonstration, always succeed
            tokio::spawn(async move {
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"mock build/test success".to_vec(),
                    stderr: vec![],
                })
            })
        }
    }

    // A dummy watcher for the mock approach
    fn create_dummy_watcher() -> RecommendedWatcher {
        // It's not actually used if we only feed events to notify_rx.
        // If you need a compile-time placeholder, do something like:
        use notify::Config;
        RecommendedWatcher::new_immediate(|_res| {}).expect("dummy watcher init")
    }

    #[tokio::test]
    async fn test_watch_and_reload_real_fs_changes() {
        // 1) Set up a minimal workspace
        let tmp_dir = tempdir().expect("create tempdir");
        let ws_path = tmp_dir.path().to_path_buf();

        // For demonstration, define a mock or minimal workspace
        // that references `ws_path`, 
        // and let's say it sees any path in ws_path/src or "Cargo.toml" as relevant.
        let workspace = MockWorkspace::new(ws_path.clone());
        
        // Create the src/ directory
        tokio::fs::create_dir_all(ws_path.join("src")).await.unwrap();
        // Possibly we have a main.rs or something to make it a real crate.

        // 2) We define a channel to collect rebuild results
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(10);

        // 3) We define a runner or use a mock runner that logs or sets a boolean on rebuild
        let runner = Arc::new(MockCommandRunner::default());

        // 4) Spawn watch_and_reload in a background task
        let cancel_token = CancellationToken::new();
        let ws_handle = tokio::spawn({
            let workspace_ref = workspace.clone();
            let runner_ref = runner.clone();
            let tx_ref = Some(tx.clone());
            let cancel_ref = cancel_token.clone();
            async move {
                workspace_ref.watch_and_reload(tx_ref, runner_ref, cancel_ref).await
            }
        });

        // 5) Actually modify a relevant file
        let cargo_toml_path = ws_path.join("Cargo.toml");
        tokio::fs::write(&cargo_toml_path, b"[package]\nname=\"test\"").await.unwrap();

        // Wait a bit for notify to pick up the change
        // The test code might require some small sleep to ensure the event is processed
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // 6) We expect watch_and_reload to rebuild or test, so we check the channel for a success or error
        if let Ok(Some(msg)) = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv()).await {
            match msg {
                Ok(_) => println!("Got a rebuild success after file change!"),
                Err(e) => println!("Got a rebuild error: {:?}", e),
            }
        } else {
            panic!("No rebuild result after changing Cargo.toml!");
        }

        // 7) Cancel
        cancel_token.cancel();
        let result = ws_handle.await.unwrap();
        assert!(result.is_ok(), "watch_and_reload ended with Ok(()) after cancellation");
    }

    // -----------------------------------------------------------------------
    // 2) Mock Approach: We do not rely on real FS changes,
    //    we feed events manually to the watch_loop or notify_rx.
    // -----------------------------------------------------------------------
    // We'll define a minimal test that simulates relevant/irrelevant changes
    // without real file watchers.

    #[tokio::test]
    async fn test_watch_and_reload_mock_events() {
        // We'll skip actual watcher creation and just replicate the logic in a simpler environment.

        // 1) Construct a workspace
        let workspace = MockWorkspace::new(PathBuf::from("/mock/workspace"));

        // 2) We'll create an async_channel for file events
        let (notify_tx, notify_rx) = async_channel::unbounded::<notify::Result<notify::Event>>();

        // 3) We'll define a (fake) watcher handle
        // In real usage, you'd store it in `_watcher`; we just keep an Option or something.
        let mut watcher = create_dummy_watcher();

        // 4) We'll define a channel to receive rebuild results
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(5);
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();

        // 5) spawn the watch_loop
        let ws_future = tokio::spawn({
            let w_ref = &workspace;
            let path = PathBuf::from("/mock/workspace");
            watch_loop(
                w_ref,
                &mut watcher,
                &path,
                notify_rx,
                Some(tx.clone()),
                runner,
                cancel_token.clone()
            )
        });

        // 6) Send a relevant event
        let ev_relevant = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(_)),
            paths: vec![PathBuf::from("/mock/workspace/Cargo.toml")],
            attrs: Default::default(),
        };
        notify_tx.send(Ok(ev_relevant)).await.unwrap();

        // Wait a bit for it to process
        if let Some(msg) = rx.recv().await {
            println!("Got a rebuild result: {:?}", msg);
        } else {
            panic!("Expected a rebuild result from relevant file event");
        }

        // 7) Send an irrelevant event
        let ev_irrelevant = notify::Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(_)),
            paths: vec![PathBuf::from("/some/unrelated/file.txt")],
            attrs: Default::default(),
        };
        notify_tx.send(Ok(ev_irrelevant)).await.unwrap();

        // We expect NO message this time, or no extra rebuild
        let maybe_msg = tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv()).await;
        assert!(maybe_msg.is_err(), "No new message for irrelevant path");

        // 8) Cancel
        cancel_token.cancel();
        let result = ws_future.await.unwrap();
        assert!(result.is_ok(), "watch_loop ended Ok after cancel");
    }

    // -----------------------------------------------------------------------
    // Additional tests:
    //  - Force an error event, check channel sees an error
    //  - Force rebuild_or_test to fail => see if channel gets an error
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_watch_and_reload_notify_error() {
        let workspace = MockWorkspace::new(PathBuf::from("/mock/ws"));
        let (notify_tx, notify_rx) = async_channel::unbounded();
        let mut watcher = create_dummy_watcher();
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(5);
        let runner = Arc::new(MockCommandRunner::default());
        let cancel_token = CancellationToken::new();

        let handle = tokio::spawn({
            let path = PathBuf::from("/mock/ws");
            watch_loop(&workspace, &mut watcher, &path, notify_rx, Some(tx.clone()), runner, cancel_token.clone())
        });

        // Send an error event
        let fake_err = notify::Error::generic("some notify error");
        notify_tx.send(Err(fake_err)).await.unwrap();

        let msg = rx.recv().await.expect("Expected an error message from watch_loop");
        assert!(msg.is_err(), "Should have an error result in the channel");

        cancel_token.cancel();
        let result = handle.await.unwrap();
        assert!(result.is_err(), "Watch ended in error from notify error");
    }

    #[tokio::test]
    async fn test_watch_and_reload_rebuild_fails() {
        // If rebuild_or_test fails, that error is sent to 'tx'.
        let workspace = MockWorkspaceWhichFails::new("/fake/path");
        //  etc...
    }
}

#[cfg(test)]
#[disable]
mod test_is_relevant_change {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_relevant_cargo_toml() {
        let ws = mock_workspace();
        let path = PathBuf::from("Cargo.toml");
        assert!(ws.is_relevant_change(&path));
    }

    #[test]
    fn test_is_relevant_src_file() {
        let ws = mock_workspace_with_crate_src("/some/crate/src");
        let path = PathBuf::from("/some/crate/src/lib.rs");
        assert!(ws.is_relevant_change(&path));
    }

    #[test]
    fn test_not_relevant() {
        let ws = mock_workspace_with_crate_src("/some/other/path/src");
        let path = PathBuf::from("/unrelated/path/main.rs");
        assert!(!ws.is_relevant_change(&path));
    }
}
