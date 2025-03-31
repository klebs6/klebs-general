// ---------------- [ File: workspacer-cli/src/validate.rs ]
crate::ix!();

/// Validate everything (or just one crate)
#[derive(Debug, StructOpt)]
pub enum ValidateSubcommand {
    /// Validate a single crate
    Crate {
        /// Path to the crate (must contain a Cargo.toml)
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    /// Validate an entire workspace
    Workspace {
        /// Path to the workspace root (must contain a Cargo.toml with [workspace])
        #[structopt(long = "path")]
        workspace_path: PathBuf,
    },
}

impl ValidateSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            ValidateSubcommand::Crate { crate_name } => {
                // 1) Build a CrateHandle from the given path
                let handle = CrateHandle::new(crate_name).await.map_err(|crate_err| {
                    // Convert CrateError -> WorkspaceError if needed:
                    WorkspaceError::CrateError(crate_err)
                })?;

                // 2) Perform the validation
                handle.validate_integrity().await.map_err(|crate_err| {
                    WorkspaceError::CrateError(crate_err)
                })?;

                println!("Crate at '{}' passed integrity checks!", crate_name.display());
            }

            ValidateSubcommand::Workspace { workspace_path } => {
                // 1) Build a Workspace from the given path
                let ws = Workspace::<PathBuf, CrateHandle>::new(workspace_path).await?;

                // 2) Validate all crates in that workspace
                ws.validate_integrity().await?;

                println!(
                    "Workspace at '{}' passed integrity checks for all member crates!",
                    workspace_path.display()
                );
            }
        }

        Ok(())
    }
}
