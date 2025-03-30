crate::ix!();

/// This new helper function `run_with_crate` parallels the existing
/// `run_with_workspace` pattern but focuses on a single crate. It:
///
/// 1) Builds a `CrateHandle` from the given `PathBuf`.
/// 2) Optionally checks Git cleanliness or other invariants (if desired).
/// 3) Hands the handle to the user-provided closure.
/// 4) Optionally re-validates the crate afterward.
///
/// We define it in full below:
#[tracing::instrument(level="trace", skip(operation))]
pub async fn run_with_crate<R, F>(
    crate_path: PathBuf,
    skip_git_check: bool,
    operation: F,
) -> Result<R, WorkspaceError>
where
    // The closure's eventual result:
    R: Send + 'static,

    // Our closure (operation) must accept `&CrateHandle` and return
    // a `Future<Output=Result<R, WorkspaceError>>`.
    F: for<'a> FnOnce(
            &'a CrateHandle,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, WorkspaceError>> + Send + 'a>>
      + Send
      + 'static,
{
    // 1) Create the CrateHandle from `crate_path`
    debug!("Creating CrateHandle from path='{}'", crate_path.display());
    let handle = CrateHandle::new(&crate_path).await.map_err(|crate_err| {
        error!(
            "Could not create CrateHandle from '{}': {:?}",
            crate_path.display(),
            crate_err
        );
        WorkspaceError::CrateError(crate_err)
    })?;

    // 2) (Optional) If you want to ensure Git is clean for single crates, you'd do so here.
    //    We might skip it or do something akin to:
    if !skip_git_check {
        // You could do something like `handle.ensure_git_clean()?` if that were in your API.
        debug!("(Skipping or not) any single-crate-level Git checks as needed.");
    }

    // 3) Hand control off to the user-provided closure.
    info!(
        "Running single-crate operation for crate path='{}'...",
        crate_path.display()
    );
    let result = {
        let future = operation(&handle);
        future.await?
    };

    // 4) Optionally validate integrity after the operation
    //    (similar to how `run_with_workspace` does `workspace.validate_integrity()`).
    info!("Validating crate integrity post-operation...");
    handle.validate_integrity().await.map_err(|err| {
        error!("Crate integrity validation failed: {:?}", err);
        WorkspaceError::CrateError(err)
    })?;
    debug!("Crate integrity is valid. Done.");

    // 5) Return whatever the closure returned
    Ok(result)
}

