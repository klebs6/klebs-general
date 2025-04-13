crate::ix!();

/// An enum representing either a “single crate” handle or an entire workspace.
/// Use `SingleOrWorkspace::detect(...)` to try loading a workspace, and if it
/// fails specifically with `WorkspaceError::ActuallyInSingleCrate {..}` we
/// fallback to a single crate handle.
pub enum SingleOrWorkspace {
    Single(CrateHandle),
    Workspace(Workspace<PathBuf, CrateHandle>),
}

impl SingleOrWorkspace {
    /// Attempt to detect if `path` is a valid workspace. 
    /// If so, return `SingleOrWorkspace::Workspace`.
    /// If the error is `ActuallyInSingleCrate`, fallback to single crate.
    /// Otherwise, return the original error.
    #[tracing::instrument(level = "trace")]
    pub async fn detect(path: &Path) -> Result<Self, WorkspaceError> {
        // To handle relative vs. absolute paths, let's canonicalize if possible:
        let path_canon = match tokio::fs::canonicalize(path).await {
            Ok(abs) => abs,
            Err(e) => {
                error!("Could not canonicalize path={:?}: {:?}", path, e);
                // fallback to the raw path anyway, or return an error
                path.to_path_buf()
            }
        };

        debug!("Attempting to interpret path='{}' as a workspace", path_canon.display());

        match Workspace::<PathBuf, CrateHandle>::new(&path_canon).await {
            Ok(ws) => {
                debug!("Successfully loaded path='{}' as a workspace", path_canon.display());
                Ok(SingleOrWorkspace::Workspace(ws))
            }
            Err(e) => {
                match e {
                    WorkspaceError::ActuallyInSingleCrate { .. } => {
                        // Fallback: treat it as a single crate
                        debug!("Falling back to single crate handle for path='{}'", path_canon.display());
                        let ch = CrateHandle::new(&path_canon).await.map_err(|crate_err| {
                            WorkspaceError::CrateError(crate_err)
                        })?;
                        Ok(SingleOrWorkspace::Single(ch))
                    }
                    other => {
                        error!("Failed to interpret path='{}' as a workspace: {:?}", path_canon.display(), other);
                        Err(other)
                    }
                }
            }
        }
    }

    /// Check that Git is clean (if you so desire). We do so by calling
    /// `git status --porcelain` and verifying no output. This is the same
    /// logic you'd do for either single crate or entire workspace, so there's
    /// no difference inside here. 
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn ensure_git_clean(&self) -> Result<(), WorkspaceError> {
        // Just call `git status --porcelain`
        use std::process::Command;

        debug!("Running git cleanliness check in single_or_workspace ...");
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .map_err(|io_err| {
                error!("Failed to run `git status --porcelain`: {:?}", io_err);
                WorkspaceError::GitError(GitError::FailedToRunGitStatusMakeSureGitIsInstalled)
            })?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            error!(
                "Command `git status --porcelain` exited with non-zero status code: {}",
                code
            );
            return Err(WorkspaceError::GitError(
                GitError::WorkingDirectoryIsNotCleanAborting,
            ));
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        if !stdout_str.trim().is_empty() {
            error!("Git working directory is not clean:\n{}", stdout_str);
            return Err(WorkspaceError::GitError(
                GitError::WorkingDirectoryIsNotCleanAborting,
            ));
        }

        info!("Git working directory is clean; proceeding.");
        Ok(())
    }

    /// Validate integrity for either a single crate or a workspace.
    #[tracing::instrument(level="trace", skip(self))]
    pub async fn validate_integrity(&mut self) -> Result<(), WorkspaceError> {
        match self {
            SingleOrWorkspace::Single(ch) => {
                ch.validate_integrity().await.map_err(|crate_err| {
                    WorkspaceError::CrateError(crate_err)
                })
            }
            SingleOrWorkspace::Workspace(ws) => {
                ws.validate_integrity().await
            }
        }
    }

    /// A convenience function for hooking into your existing “run_with_crate”
    /// or “run_with_workspace” approach. You pass a closure that takes either
    /// a `&mut CrateHandle` or a `&mut Workspace`, depending on the variant.
    ///
    /// For example:
    /// ```ignore
    /// single_or_ws.run_operation(|handle_or_ws| async move {
    ///     match handle_or_ws {
    ///         SingleOrWorkspace::Single(ch) => {
    ///             // do crate-specific code
    ///         },
    ///         SingleOrWorkspace::Workspace(ws) => {
    ///             // do workspace code
    ///         }
    ///     }
    ///     Ok(())
    /// }).await?;
    /// ```
    #[tracing::instrument(level="trace", skip(self, operation))]
    pub async fn run_operation<R, F>(&mut self, operation: F) -> Result<R, WorkspaceError>
    where
        // Our user-defined closure's return type is `Result<R, WorkspaceError>`.
        R: Send + 'static,

        // We define an async closure trait bound:
        F: for<'a> FnOnce(
                &'a mut SingleOrWorkspace
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, WorkspaceError>> + Send + 'a>>
          + Send
          + 'static
    {
        // Just call it directly:
        let fut = operation(self);
        fut.await
    }
}
