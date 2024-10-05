crate::ix!();

impl Workspace {

    /// Rebuilds the workspace or runs tests when a change is detected.
    pub async fn rebuild_or_test(&self) -> Result<(), WorkspaceError> {

        let workspace_path = self.path();

        info!("Running cargo build...");

        let output = tokio::process::Command::new("cargo")
            .arg("build")
            .current_dir(&workspace_path)
            .output()
            .await
            .map_err(|e| BuildError::CommandError { io: e })?;

        if !output.status.success() {
            error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(WorkspaceError::from(BuildError::BuildFailed {
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }));
        }

        info!("Rebuild succeeded, running tests...");

        let test_output = tokio::process::Command::new("cargo")
            .arg("test")
            .current_dir(&workspace_path)
            .output()
            .await
            .map_err(|e| TestFailure::UnknownError {
                stdout: None,
                stderr: Some(e.to_string()),
            })?;

        if !test_output.status.success() {
            let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
            let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());

            error!("Tests failed: {:#?}", stderr);
            return Err(WorkspaceError::from(TestFailure::UnknownError { stdout, stderr }));
        }

        info!("Tests passed successfully.");
        Ok(())
    }

    /// Watches for file changes and triggers rebuilds/tests.
    pub async fn watch_and_reload(&self, sender: Option<mpsc::Sender<Result<(), WorkspaceError>>>) -> Result<(), WorkspaceError> {

        let workspace_path = self.path().clone();

        // Channel for receiving file change events
        let (notify_tx, notify_rx) = tokio::sync::mpsc::channel(100);

        // Create a `notify` file watcher
        let mut watcher = RecommendedWatcher::new(move |res| {
            let _ = notify_tx.try_send(res);
        }, notify::Config::default())
        .map_err(WatchError::NotifyError)?;

        // Watch the workspace directory recursively
        watcher
            .watch(&workspace_path, RecursiveMode::Recursive)
            .map_err(WatchError::NotifyError)?;

        let mut notify_rx_stream = ReceiverStream::new(notify_rx);

        while let Some(res) = notify_rx_stream.next().await {
            match res {
                Ok(event) => {
                    for path in event.paths.iter() {
                        if self.is_relevant_change(&path) {
                            info!("Detected change in file: {:?}", path);

                            // Trigger rebuild or tests on file change
                            let rebuild_result = self.rebuild_or_test().await;

                            if let Some(ref sender) = sender {
                                let _ = sender.send(rebuild_result).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("File watch error: {:?}", e);
                    if let Some(ref sender) = sender {
                        let _ = sender.send(Err(WorkspaceError::from(e.clone()))).await;
                    }
                    return Err(WorkspaceError::from(e));
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
