// ---------------- [ File: workspacer-watch-and-reload/src/process_notify_event.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #3: Process a single notify::Result<notify::Event>
// ------------------------------------------------------------------------
pub async fn process_notify_event<'a,X,E>(
    watched: &X,
    event:   Result<notify::Event, notify::Error>,
    tx:      Option<&mpsc::Sender<Result<(), E>>>,
    runner:  &Arc<dyn CommandRunner + Send + Sync + 'a>,
) -> Result<(), E>
where
    X: WatchAndReload<Error=E> + RebuildOrTest<Error=E>,
    E: From<WatchError>,
{
    match event {
        Ok(ev) => {
            for path in ev.paths.iter() {
                handle_path_change(watched, path, tx, runner).await?;
            }
        }
        Err(e) => {
            error!("File watch error: {:?}", e);
            let e: Arc<notify::Error> = Arc::new(e);
            let e = WatchError::NotifyError(e);
            if let Some(sender) = tx {
                let _ = sender.send(Err(E::from(e.clone()))).await;
            }
            return Err(E::from(e));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_process_notify_event {
    use super::*;
    use tracing::{info, error, debug};
    use tokio::sync::mpsc;
    use std::os::unix::process::ExitStatusExt;
    use std::sync::Arc;
    use std::path::{PathBuf, Path};
    use notify::{Event as NotifyEvent, EventKind};
    use notify::event::{ModifyKind,DataChange};
    use lightweight_command_runner::CommandRunner;

    #[derive(Clone)]
    struct MockWorkspaceAlwaysRelevant {
        root: PathBuf,
    }

    impl MockWorkspaceAlwaysRelevant {
        fn new(root: PathBuf) -> Self {
            Self { root }
        }
        fn is_relevant_change(&self, _p: &Path) -> bool {
            true
        }

        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            info!("Mock rebuild/test at {}", self.root.display());
            let handle = runner.run_command({
                let mut cmd = tokio::process::Command::new("echo");
                cmd.arg("process-notify-event-build");
                cmd
            });

            let output = handle.await.map_err(|join_err| {
                let as_io = std::io::Error::new(std::io::ErrorKind::Other, join_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(as_io),
                    context: "always-relevant mock join error".into(),
                }
            })?;

            if !output.unwrap().status.success() {
                return Err(WorkspaceError::IoError {
                    io_error: Arc::new(std::io::Error::new(
                        std::io::ErrorKind::Other, 
                        "mock build fail"
                    )),
                    context: "Mock build/test failure".into(),
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
        ) -> tokio::task::JoinHandle<Result<std::process::Output, std::io::Error>> {
            tokio::spawn(async move {
                let _ = cmd;
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"mock success".to_vec(),
                    stderr: vec![],
                })
            })
        }
    }

    // Local traits same as handle_path_change
    #[async_trait]
    trait FnRebuildOrTest {
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError>;
    }
    trait FnIsRelevant {
        fn is_relevant_change(&self, path: &Path) -> bool;
    }

    #[async_trait]
    impl FnRebuildOrTest for MockWorkspaceAlwaysRelevant {
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            self.rebuild_or_test(runner).await
        }
    }
    impl FnIsRelevant for MockWorkspaceAlwaysRelevant {
        fn is_relevant_change(&self, path: &Path) -> bool {
            self.is_relevant_change(path)
        }
    }

    // A local mock of process_notify_event
    async fn process_notify_event_mock<W: Send + Sync>(
        workspace: &W,
        event: Result<NotifyEvent, notify::Error>,
        tx: Option<&mpsc::Sender<Result<(), WorkspaceError>>>,
        runner: &Arc<dyn CommandRunner>,
    ) -> Result<(), WorkspaceError>
    where
        W: FnIsRelevant + FnRebuildOrTest
    {
        match event {
            Ok(ev) => {
                for path in ev.paths {
                    if workspace.is_relevant_change(&path) {
                        let res = workspace.rebuild_or_test(runner.as_ref()).await;
                        if let Some(sender) = tx {
                            let _ = sender.send(res).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("File watch error: {:?}", e);
                if let Some(sender) = tx {
                    let _ = sender.send(Err(WorkspaceError::IoError {
                        io_error: Arc::new(std::io::Error::new(
                            std::io::ErrorKind::Other, 
                            format!("watch error: {e}")
                        )),
                        context: "File watch error".into(),
                    })).await;
                }
                return Err(WorkspaceError::IoError {
                    io_error: Arc::new(std::io::Error::new(
                        std::io::ErrorKind::Other, 
                        format!("watch error: {e}")
                    )),
                    context: "File watch error".into(),
                });
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------
    #[traced_test]
    async fn test_process_notify_event_ok_paths() {
        info!("Starting test_process_notify_event_ok_paths");
        let workspace = MockWorkspaceAlwaysRelevant::new(PathBuf::from("/some/where"));
        let runner: Arc<dyn CommandRunner> = Arc::new(MockCommandRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let event = NotifyEvent {
            kind: EventKind::Modify(ModifyKind::Data(DataChange::Content)),
            paths: vec![PathBuf::from("Cargo.toml")],
            attrs: Default::default(),
        };

        let result = process_notify_event_mock(&workspace, Ok(event), Some(&tx), &runner).await;
        assert!(result.is_ok());

        if let Some(r) = rx.try_recv().ok() {
            assert!(r.is_ok(), "Expected Ok from rebuild");
        } else {
            panic!("No message was sent after relevant path");
        }
    }

    #[traced_test]
    async fn test_process_notify_event_err() {
        info!("Starting test_process_notify_event_err");
        let workspace = MockWorkspaceAlwaysRelevant::new(PathBuf::from("/some/path"));
        let runner: Arc<dyn CommandRunner> = Arc::new(MockCommandRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let fake_err = notify::Error::generic("some watch error");
        let result = process_notify_event_mock(&workspace, Err(fake_err), Some(&tx), &runner).await;
        assert!(result.is_err(), "Should propagate watch error");

        let msg = rx.recv().await.expect("Expected an error in channel");
        assert!(msg.is_err(), "Expected an error from watch error");
    }
}
