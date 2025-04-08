// ---------------- [ File: workspacer-run/src/run_with_workspace_and_crate_name.rs ]
crate::ix!();

/// **Key idea**: Use a higher-ranked trait bound (HRTB) so that the closure
/// can borrow `&mut workspace` and `&crate_name` for the duration of the
/// async future without lifetime conflicts.
///
/// By writing `for<'a> F: FnOnce(&'a mut Workspace<...>, &'a str) -> ... + 'a`,
/// we allow the compiler to see that "for any lifetime `'a`",
/// the closure returns a future that *only* needs `'a` to remain valid.
/// This avoids the "`'1` must outlive `'2`" error when returning the pinned future.
#[tracing::instrument(level = "trace", skip(operation))]
pub async fn run_with_workspace_and_crate_name<R, F>(
    user_supplied_path: Option<PathBuf>,
    skip_git_check: bool,
    crate_name: String,
    operation: F,
) -> Result<R, WorkspaceError>
where
    // The closure's result
    R: Send + 'static,

    // We define a higher-ranked trait bound over `'a`. This says:
    // "For any lifetime `'a`, the closure can accept
    //  `&'a mut Workspace` and `&'a str`, returning a Future
    //  that also does not outlive `'a`."
    //
    // This is crucial to allow the closure to borrow `workspace` and `crate_name`
    // for the async block without requiring `'static` references.
    F: for<'a> FnOnce(
            &'a mut Workspace<PathBuf, CrateHandle>,
            &'a str,
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
                    context: "getting current_dir in run_with_workspace_and_crate_name".to_string(),
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
    info!("Running subcommand-specific logic for crate_name='{}'...", crate_name);

    // IMPORTANT: We call `operation(...)` right here in the same scope,
    // immediately await it, *not* returning the future across function boundaries.
    let result = {
        let future = operation(&mut workspace, &crate_name);
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
        "Successfully completed subcommand for crate='{}' in workspace='{}'",
        crate_name, workspace_path.display()
    );

    // 6) Return whatever the closure's operation returned
    Ok(result)
}
