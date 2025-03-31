// ---------------- [ File: workspacer-cli/src/register.rs ]
crate::ix!();

/// Register subcommand for naming files, adding crate references, etc.
#[derive(Debug, StructOpt)]
pub enum RegisterSubcommand {
    /// Ensure all source files in a single crate are “registered”
    /// (that is, it calls `CrateHandle::ensure_all_source_files_are_registered()`).
    CrateFiles {
        /// Path (or name) of the crate directory
        #[structopt(long = "crate")]
        crate_name: PathBuf,

        /// If true, skip checking for Git cleanliness
        #[structopt(long)]
        skip_git_check: bool,
    },

    /// Ensure all source files in every crate of a workspace are “registered.”
    /// (Calls `Workspace::ensure_all_source_files_are_registered()`.)
    AllFiles {
        /// Path to the workspace root
        #[structopt(long = "path")]
        workspace_path: PathBuf,

        /// If true, skip checking for Git cleanliness
        #[structopt(long)]
        skip_git_check: bool,
    },

    /// Register a new internal crate in a “prefix group” crate
    /// by adding a `[dependencies]` entry in the prefix crate’s Cargo.toml
    /// plus a `pub use new_crate_name::*;` line in `src/lib.rs`.
    Internal {
        /// Path (or name) of the prefix crate
        #[structopt(long = "prefix-crate")]
        prefix_crate: PathBuf,

        /// Path (or name) of the new crate to add
        #[structopt(long = "new-crate")]
        new_crate: PathBuf,

        /// Optionally specify the workspace root path
        /// (defaults to current directory if omitted).
        #[structopt(long = "workspace-path")]
        workspace_path: Option<PathBuf>,

        /// If true, skip checking for Git cleanliness
        #[structopt(long)]
        skip_git_check: bool,
    },
}

impl RegisterSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // ------------------------------------------------------------
            // 1) Single crate “register” => ensure_all_source_files_are_registered
            // ------------------------------------------------------------
            RegisterSubcommand::CrateFiles {
                crate_name,
                skip_git_check,
            } => {
                // Use our helper that loads the workspace plus a specific crate
                // e.g. `run_with_workspace_and_crate_name`.
                // Then call `crate_handle.ensure_all_source_files_are_registered()`.

                let crate_name_owned = crate_name.clone();
                let skip_git = *skip_git_check;
                run_with_workspace_and_crate_name(Some(crate_name_owned), skip_git, crate_name.to_string_lossy().to_string(),
                    |ws, crate_name_str| {
                        Box::pin(async move {
                            // Find that crate in the workspace
                            let crate_option = ws.find_crate_by_name(crate_name_str).await;
                            let ch = match crate_option {
                                Some(ch) => ch,
                                None => {
                                    return Err(CrateError::CrateNotFoundInWorkspace { 
                                        crate_name: crate_name_str.to_string() 
                                    }.into());
                                }
                            };

                            let mut guard = ch.lock().await;
                            guard.ensure_all_source_files_are_registered().await?;

                            Ok(())
                        })
                    }
                ).await
            }

            // ------------------------------------------------------------
            // 2) AllFiles => entire workspace `ensure_all_source_files_are_registered`
            // ------------------------------------------------------------
            RegisterSubcommand::AllFiles {
                workspace_path,
                skip_git_check,
            } => {
                let ws_path = Some(workspace_path.clone());
                let skip_git = *skip_git_check;

                run_with_workspace(ws_path, skip_git, move |ws| {
                    Box::pin(async move {
                        ws.ensure_all_source_files_are_registered().await?;

                        Ok(())
                    })
                }).await
            }

            // ------------------------------------------------------------
            // 3) Internal => “register_in_prefix_crate” routine
            // ------------------------------------------------------------
            RegisterSubcommand::Internal {
                prefix_crate,
                new_crate,
                workspace_path,
                skip_git_check,
            } => {
                // We'll interpret `prefix_crate` and `new_crate` as *names* or *paths*
                // and do the usual pattern: load the workspace, find the two crates,
                // then call `ws.register_in_prefix_crate(&prefix_crate_handle, &new_crate_handle)`.
                let skip_git = *skip_git_check;
                let ws_path = workspace_path.clone();
                let prefix_crate_str = prefix_crate.to_string_lossy().to_string();
                let new_crate_str    = new_crate.to_string_lossy().to_string();

                run_with_workspace(ws_path, skip_git, move |ws| {
                    Box::pin(async move {
                        // We do `ws.find_crate_by_name(...)` or by path. 
                        // Or you can do the `CrateHandle::new(&prefix_crate_path).await` 
                        // if it’s not truly a name but a path. 
                        // For consistency, let's treat them as “workspace crate name”:
                        let prefix_opt = ws.find_crate_by_name(&prefix_crate_str).await;
                        let prefix_h = match prefix_opt {
                            Some(ch) => ch,
                            None => {
                                return Err(CrateError::CrateNotFoundInWorkspace { 
                                    crate_name: prefix_crate_str 
                                }.into());
                            }
                        };

                        let new_opt = ws.find_crate_by_name(&new_crate_str).await;
                        let new_h = match new_opt {
                            Some(ch) => ch,
                            None => {
                                return Err(CrateError::CrateNotFoundInWorkspace { 
                                    crate_name: new_crate_str 
                                }.into());
                            }
                        };

                        // For the call to `ws.register_in_prefix_crate`, 
                        // we need ephemeral handles of type H (not Arc<Mutex<H>>).
                        let prefix_clone = {
                            let locked = prefix_h.lock().await;
                            locked.clone()
                        };
                        let new_clone = {
                            let locked = new_h.lock().await;
                            locked.clone()
                        };

                        // Now call the library trait:
                        ws.register_in_prefix_crate(&prefix_clone, &new_clone).await
                    })
                }).await
            }
        }
    }
}
