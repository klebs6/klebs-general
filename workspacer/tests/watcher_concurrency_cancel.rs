// ---------------- [ File: tests/watcher_concurrency_cancel.rs ]
// ---------------- [ File: tests/watcher_concurrency_cancel.rs ]
// tests/watcher_concurrency_cancel.rs

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio::fs;
use tokio_util::sync::CancellationToken;
use mockall::{mock, Sequence, predicate::*};
use std::io;
use std::process::Output;

use crate::workspace::{Workspace, WatchAndReload};
use crate::errors::WorkspaceError;
use crate::mock::{create_mock_workspace, CrateConfig};
use crate::lightweight_command_runner::CommandRunner;

#[cfg(test)]
mod concurrency_cancel_tests {
    use super::*;
    // We re-declare a mock if needed, otherwise you can reuse your existing mock runner.
    mock! {
        pub CommandRunner {}

        impl CommandRunner for CommandRunner {
            fn run_command(&self, cmd: tokio::process::Command) -> tokio::task::JoinHandle<Result<Output, io::Error>>;
        }

        unsafe impl Send for CommandRunner {}
        unsafe impl Sync for CommandRunner {}
    }

    /// Test that cancellation in mid-build or mid-test stops the watch loop gracefully.
    #[tokio::test]
    async fn test_watch_and_reload_with_cancellation_midway() -> Result<(), WorkspaceError> {
        // 1) Create mock workspace
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_cancellation")
                .with_src_files()
                .with_readme(),
        ]).await?;

        // 2) Write some valid code
        let src_dir = workspace_path.join("crate_cancellation").join("src");
        fs::write(
            src_dir.join("lib.rs"), 
            r#"pub fn some_function() {}"#
        ).await?;

        // 3) Mock command runner: run_command is never completed for the second call
        //    (simulate that we are "stuck" building until cancellation happens)
        let mut runner = MockCommandRunner::new();
        let mut seq = Sequence::new();

        // First run_command returns success quickly
        runner.expect_run_command()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_cmd| {
                tokio::spawn(async {
                    Ok(Output {
                        status: std::process::ExitStatus::from_raw(0),
                        stdout: b"OK".to_vec(),
                        stderr: vec![],
                    })
                })
            });

        // Second run_command simulates an indefinite wait or a slow operation
        runner.expect_run_command()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_cmd| {
                tokio::spawn(async {
                    // Instead of finishing, we just sleep a long time
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    Ok(Output {
                        status: std::process::ExitStatus::from_raw(0),
                        stdout: b"OK".to_vec(),
                        stderr: vec![],
                    })
                })
            });

        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        // 4) Initialize workspace
        let workspace = Arc::new(Workspace::new(&workspace_path).await?);
        let cancel_token = CancellationToken::new();

        let (tx, mut rx) = mpsc::channel(10);
        let watch_task = tokio::spawn({
            let ws = Arc::clone(&workspace);
            let rnr = Arc::clone(&runner);
            let ct = cancel_token.clone();
            async move {
                ws.watch_and_reload(Some(tx), rnr, ct).await.unwrap();
            }
        });

        // 5) Trigger a relevant file change that leads to build/test #1
        fs::write(src_dir.join("lib.rs"), b"pub fn changed_again() {}").await?;

        // We expect a success message for the first trigger
        if let Ok(Some(res)) = timeout(Duration::from_secs(3), rx.recv()).await {
            assert!(res.is_ok(), "First build/test should succeed quickly");
        } else {
            panic!("No result from first file change within 3s");
        }

        // 6) Trigger a second file change, which the mock runner pretends to be stuck building
        fs::write(src_dir.join("lib.rs"), b"pub fn changed_blocking() {}").await?;

        // Now we cancel after a short delay
        tokio::time::sleep(Duration::from_millis(500)).await;
        cancel_token.cancel();

        // Wait up to 3s for the watch task to terminate
        let _ = timeout(Duration::from_secs(3), watch_task).await
            .expect("Watcher didn't exit on cancellation");

        // Because we canceled mid-build, the watch loop should have ended. 
        // We expect no second success message to appear, but we do want the watch loop to exit gracefully.

        // Since it's canceled, we won't see a second build success. 
        // Check the channel if you want to ensure we got no second success:
        let maybe_msg = rx.try_recv();
        assert!(maybe_msg.is_err(), "No additional messages expected after cancellation");

        Ok(())
    }
}