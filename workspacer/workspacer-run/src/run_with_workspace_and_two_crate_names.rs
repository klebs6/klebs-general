// ---------------- [ File: workspacer-run/src/run_with_workspace_and_two_crate_names.rs ]
crate::ix!();

/// 2) We define a new helper function that is basically the same pattern as
///    `run_with_workspace_and_crate_name`, but it takes *two* crate name strings.
///    This avoids the lifetime conflict by using a higher-ranked trait bound.
#[tracing::instrument(level = "trace", skip(operation))]
pub async fn run_with_workspace_and_two_crate_names<R, F>(
    user_supplied_path: Option<PathBuf>,
    skip_git_check: bool,
    target_crate_name: String,
    dep_crate_name: String,
    operation: F,
) -> Result<R, WorkspaceError>
where
    R: Send + 'static,
    F: for<'a> FnOnce(
            &'a mut Workspace<PathBuf, CrateHandle>,
            &'a str,  // target crate name
            &'a str,  // dep crate name
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, WorkspaceError>> + Send + 'a>>
      + Send
      + 'static,
{
    // 1) Determine the workspace directory
    let workspace_path = match user_supplied_path {
        Some(path) => {
            debug!("User supplied a workspace path='{}'", path.display());
            path
        }
        None => {
            debug!("No workspace path supplied; falling back to current directory");
            std::env::current_dir().map_err(|io_err| {
                error!("Could not get current_dir: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: std::sync::Arc::new(io_err),
                    context: "getting current_dir in run_with_workspace_and_two_crate_names".to_string(),
                }
            })?
        }
    };

    // 2) Load/create the workspace
    info!("Opening workspace at '{}'", workspace_path.display());
    let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&workspace_path).await.map_err(|e| {
        error!("Failed to create/load workspace: {:?}", e);
        e
    })?;

    // 3) Optionally check Git cleanliness
    if !skip_git_check {
        info!("Ensuring Git working directory is clean...");
        workspace.ensure_git_clean().await.map_err(|git_err| {
            error!("Git is not clean: {:?}", git_err);
            WorkspaceError::GitError(git_err)
        })?;
        debug!("Git working directory confirmed clean.");
    } else {
        debug!("Skipping Git clean check because `skip_git_check` is true.");
    }

    // 4) Hand control off to the subcommand's closure
    info!(
        "Running subcommand-specific logic for target='{}' / dep='{}' ...",
        target_crate_name, dep_crate_name
    );

    // We call `operation(...)` right here and `.await` immediately.
    let result = {
        let future = operation(&mut workspace, &target_crate_name, &dep_crate_name);
        future.await?
    };

    // 5) Validate workspace integrity afterward
    info!("Validating workspace integrity post-operation...");
    workspace.validate_integrity().await.map_err(|err| {
        error!("Workspace integrity validation failed: {:?}", err);
        err
    })?;
    debug!("Workspace integrity is valid. Done.");

    info!(
        "Successfully completed subcommand: target='{}', dep='{}' in workspace='{}'",
        target_crate_name, dep_crate_name, workspace_path.display()
    );

    // 6) Return whatever the closure's operation returned
    Ok(result)
}
