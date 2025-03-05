// ---------------- [ File: workspacer-register/src/bin/ensure_all_source_files_are_registered.rs ]
use workspacer_3p::*;
use workspacer_errors::*;
use workspacer_crate::*;
use workspacer_workspace::*;
use workspacer_register::*;
use workspacer_git::*;

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();

    let path = PathBuf::from(".");

    match Workspace::<PathBuf, CrateHandle>::new(&path).await {
        Ok(workspace) => {
            // If we successfully built a workspace, proceed.
            workspace.ensure_git_clean().await?;
            workspace.ensure_all_source_files_are_registered().await?;
            println!("workspace [ensured all source files are registered]");
        }
        Err(WorkspaceError::ActuallyInSingleCrate { path: _ }) => {
            // Fallback to single crate if `[workspace]` was not found
            println!("No [workspace] found; using single-crate logic...");

            // Build a single CrateHandle
            let single_crate = CrateHandle::new(&path).await?;

            single_crate.ensure_git_clean().await
                .map_err(|git_err| WorkspaceError::GitError(git_err))?;

            single_crate.ensure_all_source_files_are_registered().await?;

            println!("single crate [ensured all source files are registered]");
        }
        Err(e) => {
            // Some other workspace error
            eprintln!("failed with error: {:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
