// ---------------- [ File: workspacer-run/src/run_with_workspace.rs ]
crate::ix!();

/// This helper is analogous to `run_with_workspace_and_crate_name`, but it only loads
/// the workspace (optionally checking Git). No crate name needed.
#[tracing::instrument(level = "trace", skip(operation))]
pub async fn run_with_workspace<R, F>(
    user_supplied_path: Option<PathBuf>,
    skip_git_check: bool,
    operation: F,
) -> Result<R, WorkspaceError>
where
    R: Send + 'static,
    F: for<'a> FnOnce(
            &'a mut Workspace<PathBuf, CrateHandle>,
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
            debug!("No workspace path supplied; using current directory");
            std::env::current_dir().map_err(|io_err| {
                error!("Could not get current_dir: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: "getting current_dir in run_with_workspace".to_string(),
                }
            })?
        }
    };

    // 2) Load the workspace
    info!("Opening workspace at '{}'", workspace_path.display());
    let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&workspace_path).await.map_err(|e| {
        error!("Failed to open workspace: {:?}", e);
        e
    })?;

    // 3) Optionally check Git
    if !skip_git_check {
        info!("Ensuring Git working directory is clean...");
        workspace.ensure_git_clean().await.map_err(|git_err| {
            error!("Git is not clean: {:?}", git_err);
            WorkspaceError::GitError(git_err)
        })?;
    } else {
        debug!("Skipping git check because skip_git_check=true");
    }

    // 4) Run the closure immediately
    let result = {
        let fut = operation(&mut workspace);
        fut.await?
    };

    // 5) Validate integrity
    info!("Validating workspace integrity post-operation...");
    workspace.validate_integrity().await?;

    info!("Done analyzing or operating on the workspace at '{}'", workspace_path.display());
    Ok(result)
}
