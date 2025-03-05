// ---------------- [ File: workspacer-publish/src/bin/publish_public_crates_in_order.rs ]
//! This binary replicates (in Rust) the logic of a shell script that
//! publishes all public crates in a workspace to crates.io, in
//! topological order, skipping crates that are already published.

use structopt::StructOpt;
use workspacer_workspace::*;
use workspacer_publish::*;
use workspacer_errors::*;
use workspacer_crate::*;
use workspacer_3p::*;
use workspacer_git::*;

/// Command line interface for publishing public crates in topological order.
#[derive(StructOpt, Debug)]
#[structopt(name = "workspacer-publish")]
pub struct PublishPublicCratesInOrderCli {
    /// Path to the workspace root
    #[structopt(long)]
    workspace_path: Option<PathBuf>,

    #[structopt(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();

    let cli     = PublishPublicCratesInOrderCli::from_args();
    let path    = cli.workspace_path.unwrap_or(PathBuf::from("."));
    let dry_run = cli.dry_run;

    match Workspace::<PathBuf, CrateHandle>::new(&path).await {
        Ok(workspace) => {
            // If we successfully built a workspace, proceed.
            workspace.ensure_git_clean().await?;
            workspace.try_publish(dry_run).await?;
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

            single_crate.try_publish(dry_run).await
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
