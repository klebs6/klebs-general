// ---------------- [ File: workspacer-cli/src/watch.rs ]
crate::ix!();

/// Watch for changes in a single crate or the entire workspace,
/// rebuilding/testing on file changes until canceled.
#[derive(Debug, StructOpt)]
pub enum WatchSubcommand {
    /// Watch a single crate
    Crate {
        /// Path to the crate directory (must contain Cargo.toml)
        #[structopt(long = "crate")]
        crate_name: std::path::PathBuf,
    },
    /// Watch an entire workspace
    Workspace {
        /// Path to the workspace root (must contain Cargo.toml with [workspace])
        #[structopt(long = "path")]
        workspace_path: std::path::PathBuf,
    },
}

impl WatchSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // -----------------------------------------------------------
            // 1) Single-crate watch
            // -----------------------------------------------------------
            WatchSubcommand::Crate { crate_name } => {
                // (a) Build a CrateHandle from the user-specified path
                let single_crate = CrateHandle::new(crate_name)
                    .await
                    .map_err(WorkspaceError::CrateError)?;

                // (b) Build or pick a CommandRunner (here, minimal).
                let runner = Arc::new(DefaultCommandRunner);

                // (c) Create a cancellation token, so we can stop the watch loop if desired.
                let cancel_token = CancellationToken::new();

                // (d) Optional: Create a channel for receiving watch events or rebuild results.
                let (tx, mut rx) = mpsc::channel::<Result<(), CrateError>>(10);

                // (e) Start watching until canceled. This call blocks while watching changes.
                //     You might want to spawn it in a background task instead.
                single_crate
                    .watch_and_reload(Some(tx), runner, cancel_token.clone())
                    .await
                    .map_err(WorkspaceError::CrateError)?;

                // If you want to handle incoming rebuild/test results, you can do so:
                // e.g. in a separate task or after it returns. For simplicity, do nothing here.

                Ok(())
            }

            // -----------------------------------------------------------
            // 2) Entire workspace watch
            // -----------------------------------------------------------
            WatchSubcommand::Workspace { workspace_path } => {
                // (a) Build the workspace from the user-specified path
                let ws = Workspace::<std::path::PathBuf, CrateHandle>::new(workspace_path).await?;

                // (b) Build a CommandRunner (use real or mock).
                let runner = Arc::new(DefaultCommandRunner);

                // (c) Create a cancellation token
                let cancel_token = CancellationToken::new();

                // (d) Optional: Create a channel
                let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(10);

                // (e) Watch & reload until canceled
                ws.watch_and_reload(Some(tx), runner, cancel_token.clone()).await?;

                // Optionally read from `rx` to see rebuild/test results
                // e.g., in a loop or background task.

                Ok(())
            }
        }
    }
}

