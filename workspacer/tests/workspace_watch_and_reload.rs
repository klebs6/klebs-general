// ---------------- [ File: tests/workspace_watch_and_reload.rs ]
use disable_macro::disable;

#[disable]
mod workspace_watch_and_reload_tests {
    use tokio::fs;
    use std::time::Duration;
    use tracing::info;
    use tokio::sync::mpsc;
    use std::sync::Arc;
    use tokio::time::timeout;
    use mockall::{mock, predicate::*, Sequence};
    use tokio::process::Command;
    use tokio::task::JoinHandle;
    use std::process::Output;
    use std::io;
    use tokio_util::sync::CancellationToken;
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};

    // Helper functions to create ExitStatus
    #[cfg(unix)]
    fn make_exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    fn make_exit_status(code: u32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }

    // Define the mock CommandRunner within the test module
    mock! {
        pub CommandRunner {}

        impl CommandRunner for CommandRunner {
            fn run_command(&self, cmd: Command) -> JoinHandle<Result<Output, io::Error>>;
        }

        unsafe impl Send for CommandRunner {}
        unsafe impl Sync for CommandRunner {}
    }

    // Helper function to create a CommandRunner that allows any number of run_command calls
    fn create_mock_runner(
        expected_calls: usize,
        statuses: Vec<std::process::ExitStatus>,
        stdouts: Vec<Vec<u8>>,
        stderrs: Vec<Vec<u8>>,
    ) -> MockCommandRunner {
        let mut runner = MockCommandRunner::new();
        let mut seq = Sequence::new();

        for i in 0..expected_calls {
            let status = statuses[i];
            let stdout = stdouts[i].clone();
            let stderr = stderrs[i].clone();

            runner
                .expect_run_command()
                .times(1)
                .in_sequence(&mut seq)
                .returning(move |_cmd| {
                    tokio::spawn(
                        {
                            let stdout_value = stdout.clone();
                            let stderr_value = stderr.clone();
                            async move {
                                Ok(Output {
                                    status,
                                    stdout: stdout_value,
                                    stderr: stderr_value,
                                })
                            }
                        }
                    )
                });
        }

        runner
    }

    #[tokio::test]
    async fn test_watch_and_reload_on_irrelevant_file_change() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace");

        // Create a mock workspace with a simple crate
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
        ])
        .await?;

        // Create a mock command runner
        let runner = MockCommandRunner::new();

        // Wrap the runner in an Arc and cast to dyn CommandRunner
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;
        let workspace = Arc::new(workspace);

        // Create a cancellation token
        let cancel_token = CancellationToken::new();

        // Capture the output for verification
        let (tx, mut rx) = mpsc::channel(1);

        let watch_handle = tokio::spawn({
            let workspace = Arc::clone(&workspace);
            let runner = Arc::clone(&runner);
            let cancel_token = cancel_token.clone();
            async move {
                workspace
                    .watch_and_reload(Some(tx), runner, cancel_token)
                    .await
                    .unwrap();
            }
        });

        info!("Simulating a file change in README.md");

        // Simulate a file change in an irrelevant file (README.md)
        let readme_file = workspace_path.join("crate_a").join("README.md");
        fs::write(readme_file, "# Updated README")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Wait to see if any messages are received
        let duration = Duration::from_secs(2);
        match timeout(duration, rx.recv()).await {
            Ok(Some(_)) => panic!(
                "Expected no rebuild or test to be triggered for irrelevant file changes"
            ),
            Ok(None) => panic!("Channel closed unexpectedly"),
            Err(_) => {
                // No message received within the timeout duration, which is expected
                assert!(true);
            }
        }

        // Signal cancellation to stop the watcher
        cancel_token.cancel();

        // Wait for the watcher task to finish
        watch_handle.await.unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_and_reload_with_successful_rebuild_and_tests() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace");

        // Create a mock workspace with valid code and tests
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
        ])
        .await?;

        info!("Writing valid code to src/lib.rs");

        // Write valid code and tests for the crate
        let src_dir = workspace_path.join("crate_a").join("src");
        fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub fn greet() {
                    println!("Hello, world!");
                }
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn test_greet() {
                        super::greet();
                    }
                }
            "#,
        )
        .await
        .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Create a mock command runner
        // Allowing for potential multiple calls due to multiple file events
        let runner = create_mock_runner(
            2, // Expected number of run_command calls
            vec![make_exit_status(0), make_exit_status(0)], // Build and test succeed
            vec![Vec::new(), Vec::new()], // stdout
            vec![Vec::new(), Vec::new()], // stderr
        );

        // Wrap the runner in an Arc and cast to dyn CommandRunner
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        let workspace = Workspace::new(&workspace_path).await?;
        let workspace = Arc::new(workspace);

        // Create a cancellation token
        let cancel_token = CancellationToken::new();

        // Capture the output for verification
        let (tx, mut rx) = mpsc::channel(10); // Increased channel capacity

        let watch_handle = tokio::spawn({
            let workspace = Arc::clone(&workspace);
            let runner = Arc::clone(&runner);
            let cancel_token = cancel_token.clone();
            async move {
                workspace
                    .watch_and_reload(Some(tx), runner, cancel_token)
                    .await
                    .unwrap();
            }
        });

        info!("Simulating a file change in src/lib.rs");

        // Simulate a file change in the src directory
        fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub fn greet_updated() {
                    println!("Updated!");
                }
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn test_greet_updated() {
                        super::greet_updated();
                    }
                }
            "#,
        )
        .await
        .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Wait for the change event to be processed
        let result = timeout(Duration::from_secs(5), rx.recv()).await;

        if let Ok(Some(result)) = result {
            assert!(
                result.is_ok(),
                "Expected a successful rebuild and test run"
            );
        } else {
            panic!("Did not receive any rebuild or test trigger");
        }

        // Signal cancellation to stop the watcher
        cancel_token.cancel();

        // Wait for the watcher task to finish
        watch_handle.await.unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_and_reload_with_failed_rebuild() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace");

        // Create a mock workspace with invalid code
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
        ])
        .await?;

        info!("Writing invalid code to src/lib.rs");

        // Write invalid code to cause a rebuild failure
        let src_dir = workspace_path.join("crate_a").join("src");
        fs::write(src_dir.join("lib.rs"), "fn invalid_code {")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Create a mock command runner
        // Allowing for potential multiple calls due to multiple file events
        let runner = create_mock_runner(
            1, // Expected number of run_command calls (only build command since build fails)
            vec![make_exit_status(1)], // Build fails
            vec![Vec::new()],          // stdout
            vec![b"Build failed".to_vec()], // stderr
        );

        // Wrap the runner in an Arc and cast to dyn CommandRunner
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        let workspace = Workspace::new(&workspace_path).await?;
        let workspace = Arc::new(workspace);

        // Create a cancellation token
        let cancel_token = CancellationToken::new();

        // Capture the output for verification
        let (tx, mut rx) = mpsc::channel(10); // Increased channel capacity

        let watch_handle = tokio::spawn({
            let workspace = Arc::clone(&workspace);
            let runner = Arc::clone(&runner);
            let cancel_token = cancel_token.clone();
            async move {
                workspace
                    .watch_and_reload(Some(tx), runner, cancel_token)
                    .await
                    .unwrap();
            }
        });

        // Simulate a file change to trigger the watcher
        fs::write(src_dir.join("lib.rs"), "fn invalid_code {")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Wait for the change event to be processed
        let result = timeout(Duration::from_secs(5), rx.recv()).await;

        if let Ok(Some(result)) = result {
            assert!(
                matches!(result, Err(WorkspaceError::BuildError(_))),
                "Expected a build failure"
            );
        } else {
            panic!("Did not receive any rebuild or test trigger");
        }

        // Signal cancellation to stop the watcher
        cancel_token.cancel();

        // Wait for the watcher task to finish
        watch_handle.await.unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_and_reload_with_failed_tests() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace");

        // Create a mock workspace with a failing test
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
        ])
        .await?;

        info!("Writing code with a failing test");

        // Write valid code but with a test that always fails
        let src_dir = workspace_path.join("crate_a").join("src");
        fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub fn always_fail() {
                    panic!("This always fails");
                }
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn test_fail() {
                        super::always_fail();
                    }
                }
            "#,
        )
        .await
        .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Create a mock command runner
        // Allowing for potential multiple calls due to multiple file events
        let runner = create_mock_runner(
            2, // Expected number of run_command calls (build and test)
            vec![make_exit_status(0), make_exit_status(1)], // Build succeeds, test fails
            vec![Vec::new(), Vec::new()],                   // stdout
            vec![Vec::new(), b"Test failed".to_vec()],      // stderr
        );

        // Wrap the runner in an Arc and cast to dyn CommandRunner
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        let workspace = Workspace::new(&workspace_path).await?;
        let workspace = Arc::new(workspace);

        // Create a cancellation token
        let cancel_token = CancellationToken::new();

        // Capture the output for verification
        let (tx, mut rx) = mpsc::channel(10); // Increased channel capacity

        let watch_handle = tokio::spawn({
            let workspace = Arc::clone(&workspace);
            let runner = Arc::clone(&runner);
            let cancel_token = cancel_token.clone();
            async move {
                workspace
                    .watch_and_reload(Some(tx), runner, cancel_token)
                    .await
                    .unwrap();
            }
        });

        info!("Simulating a file change in src/lib.rs");

        // Simulate a file change in the src directory
        fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub fn always_fail() {
                    panic!("This always fails");
                }
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn test_fail() {
                        super::always_fail();
                    }
                }
            "#,
        )
        .await
        .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Wait for the change event to be processed
        let result = timeout(Duration::from_secs(5), rx.recv()).await;

        if let Ok(Some(result)) = result {
            assert!(
                matches!(result, Err(WorkspaceError::TestFailure(_))),
                "Expected a test failure"
            );
        } else {
            panic!("Did not receive any rebuild or test trigger");
        }

        // Signal cancellation to stop the watcher
        cancel_token.cancel();

        // Wait for the watcher task to finish
        watch_handle.await.unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_and_reload_on_relevant_file_change() -> Result<(), WorkspaceError> {
        info!("Creating a mock workspace");

        // Create a mock workspace with a simple crate
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crate_a")
                .with_src_files()
                .with_readme(),
        ])
        .await?;

        // Create a mock command runner
        // Allowing for potential multiple calls due to multiple file events
        let runner = create_mock_runner(
            2, // Expected number of run_command calls (build and test)
            vec![make_exit_status(0), make_exit_status(0)], // Build and test succeed
            vec![Vec::new(), Vec::new()],                   // stdout
            vec![Vec::new(), Vec::new()],                   // stderr
        );

        // Wrap the runner in an Arc and cast to dyn CommandRunner
        let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(runner);

        // Initialize the workspace
        let workspace = Workspace::new(&workspace_path).await?;
        let workspace = Arc::new(workspace);

        // Create a cancellation token
        let cancel_token = CancellationToken::new();

        // Capture the output for verification
        let (tx, mut rx) = mpsc::channel(10); // Increased channel capacity
        let watch_handle = tokio::spawn({
            let workspace = Arc::clone(&workspace);
            let runner = Arc::clone(&runner);
            let cancel_token = cancel_token.clone();
            async move {
                workspace
                    .watch_and_reload(Some(tx), runner, cancel_token)
                    .await
                    .unwrap();
            }
        });

        info!("Simulating a file change in src/lib.rs");

        // Simulate a file change in the src/ directory
        let src_dir = workspace_path.join("crate_a").join("src");
        fs::write(src_dir.join("lib.rs"), "fn updated_function() {}")
            .await
            .map_err(|e| FileError::WriteError { io: e.into() })?;

        // Wait for the change event to be processed
        let result = timeout(Duration::from_secs(5), rx.recv()).await;

        if let Ok(Some(result)) = result {
            assert!(result.is_ok(), "Expected a rebuild and test to be triggered");
        } else {
            panic!("Did not receive any rebuild or test trigger");
        }

        // Signal cancellation to stop the watcher
        cancel_token.cancel();

        // Wait for the watcher task to finish
        watch_handle.await.unwrap();

        Ok(())
    }
}

