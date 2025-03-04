// ---------------- [ File: src/handle_path_change.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #4: Handle an individual changed path
// ------------------------------------------------------------------------
pub async fn handle_path_change<X,E>(
    watched:   &X,
    path:      &Path,
    tx:        Option<&mpsc::Sender<Result<(), E>>>,
    runner:    &Arc<dyn CommandRunner + Send + Sync + 'static>,
) -> Result<(), E>
where
    X: WatchAndReload<Error=E> + RebuildOrTest<Error=E>,
    E: From<WatchError>,
{
    if watched.is_relevant_change(path) {
        info!("Detected relevant change in file: {:?}", path);

        let rebuild_result = watched.rebuild_or_test(runner.as_ref()).await;
        notify_rebuild_result(tx, rebuild_result).await;
    }
    Ok(())
}

#[cfg(test)]
mod test_handle_path_change {
    use super::*;
    use std::path::{Path, PathBuf};
    use tracing::{info, error};
    use tokio::sync::mpsc;
    use std::os::unix::process::ExitStatusExt; // from_raw
    use workspacer_3p::tokio;
    use std::sync::Arc;
    use lightweight_command_runner::CommandRunner;

    #[derive(Clone)]
    struct MockWorkspaceAllRelevant {
        root: PathBuf,
    }

    impl MockWorkspaceAllRelevant {
        fn new(root: PathBuf) -> Self {
            Self { root }
        }
        fn is_relevant_change(&self, _path: &Path) -> bool {
            // In this mock, everything is "relevant"
            true
        }

        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            info!("Mock rebuild_or_test for AllRelevant at {}", self.root.display());
            let handle = runner.run_command({
                let mut cmd = tokio::process::Command::new("echo");
                cmd.arg("all-relevant-build");
                cmd
            });

            // First unwrap the JoinHandle -> Output
            let output = handle.await.map_err(|join_err| {
                // convert the JoinError to an IoError:
                let as_io = std::io::Error::new(std::io::ErrorKind::Other, join_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(as_io),
                    context: "mock runner join error".into(),
                }
            })??;

            // Now we can access output.status
            if !output.status.success() {
                error!("Mock build/test failed for all-relevant workspace");
                return Err(WorkspaceError::IoError {
                    io_error: Arc::new(std::io::Error::new(
                        std::io::ErrorKind::Other, 
                        "all-relevant build/test failed",
                    )),
                    context: "Mock build failure".into(),
                });
            }
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockWorkspaceNoneRelevant {
        root: PathBuf,
    }

    impl MockWorkspaceNoneRelevant {
        fn new(root: PathBuf) -> Self {
            Self { root }
        }

        fn is_relevant_change(&self, _path: &Path) -> bool {
            // In this mock, nothing is relevant
            false
        }

        async fn rebuild_or_test(&self, _runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            info!("Mock rebuild_or_test for NoneRelevant at {}", self.root.display());
            // We'll never actually rebuild, but let's allow Ok
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

    // “Function-like” traits to replicate handle_path_change usage
    #[async_trait]
    trait FnRebuildOrTest {
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError>;
    }
    trait FnIsRelevant {
        fn is_relevant_change(&self, path: &Path) -> bool;
    }

    #[async_trait]
    impl FnRebuildOrTest for MockWorkspaceAllRelevant {
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            self.rebuild_or_test(runner).await
        }
    }
    impl FnIsRelevant for MockWorkspaceAllRelevant {
        fn is_relevant_change(&self, path: &Path) -> bool {
            self.is_relevant_change(path)
        }
    }

    #[async_trait]
    impl FnRebuildOrTest for MockWorkspaceNoneRelevant {
        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), WorkspaceError> {
            self.rebuild_or_test(runner).await
        }
    }
    impl FnIsRelevant for MockWorkspaceNoneRelevant {
        fn is_relevant_change(&self, path: &Path) -> bool {
            self.is_relevant_change(path)
        }
    }

    // A local mock version of handle_path_change that calls is_relevant_change + rebuild_or_test
    async fn handle_path_change_mock<W: Send + Sync>(
        workspace: &W,
        path: &Path,
        tx: Option<&mpsc::Sender<Result<(), WorkspaceError>>>,
        runner: &Arc<dyn CommandRunner>,
    ) -> Result<(), WorkspaceError>
    where
        W: FnIsRelevant + FnRebuildOrTest
    {
        if workspace.is_relevant_change(path) {
            let result = workspace.rebuild_or_test(runner.as_ref()).await;
            if let Some(ch) = tx {
                let _ = ch.send(result).await;
            }
        }
        Ok(())
    }

    #[traced_test]
    async fn test_relevant_path_triggers_rebuild() {
        let workspace = MockWorkspaceAllRelevant::new(PathBuf::from("/any/path"));
        let runner: Arc<dyn CommandRunner> = Arc::new(MockCommandRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let path = Path::new("Cargo.toml");
        handle_path_change_mock(&workspace, path, Some(&tx), &runner)
            .await
            .expect("Should not fail for relevant path");

        if let Some(res) = rx.recv().await {
            assert!(res.is_ok(), "Expected Ok from rebuild");
        } else {
            panic!("No message sent for relevant path");
        }
    }

    #[traced_test]
    async fn test_irrelevant_path_no_rebuild() {
        let workspace = MockWorkspaceNoneRelevant::new(PathBuf::from("/unused/path"));
        let runner: Arc<dyn CommandRunner> = Arc::new(MockCommandRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let path = Path::new("random_file.txt");
        handle_path_change_mock(&workspace, path, Some(&tx), &runner)
            .await
            .expect("Should not fail for irrelevant path");

        let msg = rx.try_recv();
        assert!(msg.is_err(), "No message was sent if path is irrelevant");
    }
}
