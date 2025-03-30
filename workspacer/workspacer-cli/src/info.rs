// ---------------- [ File: workspacer-cli/src/info.rs ]
crate::ix!();

/// Print general info about the workspace or crate
#[derive(Debug, StructOpt)]
pub enum InfoSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl InfoSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!("Entering InfoSubcommand::run with {:?}", self);

        match self {
            InfoSubcommand::Crate { crate_name } => {
                info!("Gathering crate info for path='{}'", crate_name.display());

                // 1) Build a CrateHandle from the given path
                let handle = CrateHandle::new(&crate_name.clone())
                    .await
                    .map_err(|crate_err| {
                        error!("Failed to create CrateHandle for '{}': {:?}", crate_name.display(), crate_err);
                        WorkspaceError::CrateError(crate_err)
                    })?;

                // 2) Basic info: name, version, private or not
                let name = handle.name();
                debug!("Crate '{}' => retrieved name='{}'", crate_name.display(), name);

                let version = handle.version().map_err(|err| {
                    error!("Crate '{}' => cannot get version: {:?}", name, err);
                    WorkspaceError::CrateError(err)
                })?;

                let is_priv = handle.is_private().await.map_err(|err| {
                    error!("Crate '{}' => is_private check failed: {:?}", name, err);
                    WorkspaceError::CrateError(err)
                })?;

                println!("Crate path='{}'", crate_name.display());
                println!("  name='{}'", name);
                println!("  version='{}'", version);
                println!("  private?={}", is_priv);

                // 3) Check if tests/ directory is present, list test files if so
                if handle.has_tests_directory() {
                    let test_files = handle.test_files().await.map_err(|err| {
                        error!("Crate '{}' => test_files() failed: {:?}", name, err);
                        WorkspaceError::CrateError(err)
                    })?;
                    info!("Crate '{}' => found {} test file(s)", name, test_files.len());
                    println!("  tests directory is present. Test files: {:?}", test_files);
                } else {
                    println!("  no tests/ directory found.");
                }

                // 4) Check for README
                match handle.readme_path().await {
                    Ok(Some(readme)) => {
                        info!("Crate '{}' => found README at '{}'", name, readme.display());
                        println!("  README present at '{}'", readme.display());
                    }
                    Ok(None) => {
                        warn!("Crate '{}' => no README.md found", name);
                        println!("  no README.md present");
                    }
                    Err(e) => {
                        error!("Crate '{}' => error checking readme_path: {:?}", name, e);
                        return Err(WorkspaceError::CrateError(e));
                    }
                }

                // 5) Optionally, gather bin target names if any
                match handle.gather_bin_target_names().await {
                    Ok(bin_targets) if !bin_targets.is_empty() => {
                        debug!("Crate '{}' => found bin targets: {:?}", name, bin_targets);
                        println!("  bin targets: {:?}", bin_targets);
                    }
                    Ok(_) => {
                        println!("  no [bin] targets found");
                    }
                    Err(e) => {
                        error!("Crate '{}' => gather_bin_target_names failed: {:?}", name, e);
                        return Err(WorkspaceError::CrateError(e));
                    }
                }

                info!("Finished printing info for crate='{}'", name);
                Ok(())
            },

            InfoSubcommand::Workspace { path } => {
                info!("Gathering workspace info at path='{}'", path.display());

                // 1) Build a Workspace from the given path
                let ws = Workspace::<PathBuf, CrateHandle>::new(path).await.map_err(|err| {
                    error!("Failed to create Workspace at '{}': {:?}", path.display(), err);
                    err
                })?;

                // 2) Print basic workspace details
                let crate_count = ws.n_crates();
                println!("Workspace at '{}':", path.display());
                println!("  number of crates: {}", crate_count);

                // 3) Print each crate’s name, version, etc.
                //    We’ll do a quick pass: name + (private?).
                //    If you want more details, you can do an additional lock + gather info.
                let all_names = ws.get_all_crate_names().await;
                info!(
                    "Workspace '{}' => crate names: {:?}",
                    path.display(),
                    all_names
                );

                println!("Crates in this workspace:");
                for crate_name in all_names {
                    // We can attempt to find the crate quickly and check if it’s private
                    if let Some(arc_crate) = ws.find_crate_by_name(&crate_name).await {
                        let guard = arc_crate.lock().await;
                        let ver = match guard.version() {
                            Ok(v) => v.to_string(),
                            Err(e) => {
                                warn!("Crate '{}' => cannot retrieve version: {:?}", crate_name, e);
                                "(unknown)".to_string()
                            }
                        };
                        let priv_status = match guard.is_private().await {
                            Ok(b) => b,
                            Err(e) => {
                                warn!("Crate '{}' => is_private() errored: {:?}", crate_name, e);
                                false
                            }
                        };
                        println!(
                            "  - name='{}', version='{}', private?={}",
                            crate_name, ver, priv_status
                        );
                    } else {
                        println!("  - name='{}' => error finding crate handle", crate_name);
                    }
                }

                // That’s enough for a “general info” view.
                info!("Completed workspace info for path='{}'", path.display());
                Ok(())
            }
        }
    }
}
