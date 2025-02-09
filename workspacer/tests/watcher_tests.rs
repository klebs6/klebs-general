// ---------------- [ File: tests/watcher_tests.rs ]
// tests/watcher_tests.rs

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio::fs;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use std::process::Output;
use std::io;
use mockall::{mock, Sequence, predicate::*};
use crate::command_runner::CommandRunner;
use crate::workspace::{Workspace, WatchAndReload};
use crate::errors::WorkspaceError;
use crate::mock::{create_mock_workspace, CrateConfig};

#[cfg(test)]
mod concurrency_tests {
    use super::*;

    mock! {
        pub CommandRunner {}

        impl CommandRunner for CommandRunner {
            fn run_command(&self, cmd: tokio::process::Command) -> JoinHandle<Result<Output, io::Error>>;
        }

        // Safe to mark these as Send/Sync for test usage:
        unsafe impl Send for CommandRunner {}
        unsafe impl Sync for CommandRunner {}
    }

    #[tokio::test]
    async fn test_multiple_quick_succession_file_changes() -> Result<(), WorkspaceError> {
        // 1) Create a mock workspace
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_succession")
                .with_src_files()
                .with_readme(),
        ]).await?;

        // 2) Write valid code to ensure the build/test passes
        let src_path = workspace_path.join("crate_succession").join("src").join("lib.rs");
        fs::write(
            &src_path,
            r#"
               pub fn example() {}
               #[cfg(test)]
               mod tests {
                   #[test]
                   fn test_example() {
                       super::example();
                   }
               }
            "#,
        )
        .await?;

        // 3) Mock CommandRunner set up
        let mut runner = MockCommandRunner::new();
        let mut seq = Sequence::new();

        // We'll allow 4 calls (two distinct changes, each triggers build + test).
        // Adjust if your watch_and_reload only does 1 call, but typically it does 2 (build + test).
        for _ in 0..4 {
            runner
                .expect_run_command()
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
        }
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        // 4) Create and run workspace
        let workspace = Arc::new(Workspace::new(&workspace_path).await?);

        // 5) Start watcher
        let cancel_token = CancellationToken::new();
        let (tx, mut rx) = mpsc::channel(4);

        let watch_task = tokio::spawn({
            let ws = Arc::clone(&workspace);
            let rnr = Arc::clone(&runner);
            let ctoken = cancel_token.clone();
            async move {
                ws.watch_and_reload(Some(tx), rnr, ctoken).await.unwrap();
            }
        });

        // 6) Rapidly simulate 2 changes to src/lib.rs
        //    Each one should trigger build/test
        for _ in 0..2 {
            fs::write(&src_path, b"pub fn changed_slightly() {}").await?;
            // Sleep a tiny bit so that the filesystem events trigger
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 7) Collect results. Expect 2 success results (each from build + test).
        //    If your code sends 1 message per entire “rebuild_and_test,” expect 2 total.
        let mut successes = 0;
        for _ in 0..2 {
            // Wait up to 3s for each
            if let Ok(Some(res)) = timeout(Duration::from_secs(3), rx.recv()).await {
                if res.is_ok() {
                    successes += 1;
                }
            }
        }

        // 8) Cancel the watcher and verify
        cancel_token.cancel();
        watch_task.await.unwrap();

        // If we expected 2 successful triggers:
        assert_eq!(successes, 2, "Expected 2 successful triggers (each change => build/test).");

        Ok(())
    }
}
