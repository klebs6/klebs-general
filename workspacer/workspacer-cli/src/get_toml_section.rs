// ---------------- [ File: workspacer-cli/src/get_toml_section.rs ]
crate::ix!();

/// This function handles the `ws get toml --section <SECTION> [--crate <NAME>]` logic:
///
/// - If `crate_name` is `Some(...)`, we load that crate’s `Cargo.toml` and attempt
///   to retrieve the specified `<SECTION>` from it, printing the result.
/// - If `crate_name` is `None`, we load the *workspace’s root* `Cargo.toml`
///   (i.e., `<workspace_root>/Cargo.toml`) and retrieve the specified `<SECTION>`.
#[tracing::instrument(level = "trace", skip(section, crate_name))]
pub async fn get_toml_section_flow(
    section: String,
    crate_name: Option<String>,
) -> Result<(), WorkspaceError> {
    // We'll reuse `run_with_workspace(...)` so we load the workspace from the current dir
    // (or whichever default logic that function provides).
    // We won't do a Git check here, to keep it simple. If you want a Git check, adapt as needed.
    run_with_workspace(None, false, move |ws| {
        Box::pin(async move {
            match crate_name {
                Some(ref name) => {
                    info!("Fetching TOML section='{}' for crate='{}' ...", section, name);

                    // 1) Attempt to find the crate by name
                    let arc_crate = ws.find_crate_by_name(name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace for 'get toml' subcommand", name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: name.to_owned(),
                        }
                    })?;

                    // 2) Lock to retrieve the CargoToml handle (or parse it as needed)
                    let handle = arc_crate.lock().await.clone();
                    let cargo_toml = handle.cargo_toml();

                    let guard = cargo_toml.lock().await;

                    // 3) Attempt to retrieve the specified `section` from `cargo_toml.content()`
                    let top_level = guard.get_content();

                    if let Some(value) = top_level.get(&section) {
                        // We’ll pretty-print this chunk
                        let rendered = toml::to_string_pretty(value)?;
                        println!("[{section}] in crate='{name}':\n{}", rendered);
                    } else {
                        warn!(
                            "Section '{}' not found in Cargo.toml for crate='{}'. Printing top-level content instead.",
                            section, name
                        );
                        // Optional fallback: just show entire top-level if user wants
                        let entire = toml::to_string_pretty(top_level)?;
                        println!("No section named '{}'. Full Cargo.toml:\n{}", section, entire);
                    }
                }
                None => {
                    info!("Fetching TOML section='{}' from the WORKSPACE root Cargo.toml ...", section);

                    // 1) We assume that `<workspace_root>/Cargo.toml` is the top-level
                    let workspace_root = ws.as_ref();
                    let root_cargo_toml_path = workspace_root.join("Cargo.toml");

                    // 2) Parse or load that file
                    debug!("Reading root Cargo.toml at: {:?}", root_cargo_toml_path);
                    if !root_cargo_toml_path.exists() {
                        error!("No Cargo.toml at workspace root. Searched: {:?}", root_cargo_toml_path);
                        return Err(WorkspaceError::IoError {
                            io_error: Arc::new(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "workspace root Cargo.toml not found",
                            )),
                            context: format!("Missing file at {:?}", root_cargo_toml_path),
                        });
                    }
                    let root_handle = CargoToml::new_sync(&root_cargo_toml_path)?;

                    // 3) Retrieve the specified `section` from the root’s top-level TOML
                    let top_level = root_handle.content();
                    if let Some(value) = top_level.get(&section) {
                        let rendered = toml::to_string_pretty(value)?;
                        println!("[{section}] in workspace root Cargo.toml:\n{}", rendered);
                    } else {
                        warn!(
                            "Section '{}' not found in the workspace root Cargo.toml. Printing full contents instead.",
                            section
                        );
                        let entire = toml::to_string_pretty(top_level)?;
                        println!("No section named '{}'. Full Cargo.toml:\n{}", section, entire);
                    }
                }
            }
            Ok(())
        })
    })
    .await
}
