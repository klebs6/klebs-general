// ---------------- [ File: src/bin/pin_wildcard_deps.rs ]
//! src/bin/pin_wildcards.rs
use workspacer_3p::*;
use workspacer_errors::*;
use workspacer_crate::*;
use workspacer_workspace::*;
use workspacer_pin::*;
use workspacer_git::*;

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();

    let path = PathBuf::from(".");

    match Workspace::<PathBuf, CrateHandle>::new(&path).await {
        Ok(workspace) => {
            // If we successfully built a workspace, proceed.
            workspace.ensure_git_clean().await?;
            workspace.pin_all_wildcard_dependencies().await?;
            println!("Successfully pinned wildcard dependencies in the workspace!");
        }
        Err(WorkspaceError::ActuallyInSingleCrate { path: _ }) => {
            // Fallback to single crate if `[workspace]` was not found
            println!("No [workspace] found; using single-crate logic...");

            // Build a single CrateHandle
            let single_crate = CrateHandle::new(&path).await
                .map_err(|e| WorkspaceError::CrateError(e))?;

            single_crate.ensure_git_clean().await
                .map_err(|git_err| WorkspaceError::GitError(git_err))?;

            single_crate.pin_all_wildcard_dependencies().await
                .map_err(|e| WorkspaceError::CrateError(e))?;

            println!("Successfully pinned wildcard dependencies in single crate!");
        }
        Err(e) => {
            // Some other workspace error
            eprintln!("Workspace creation failed with error: {:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
