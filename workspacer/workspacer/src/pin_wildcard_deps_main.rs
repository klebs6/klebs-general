//! src/bin/pin_wildcards.rs
use workspacer_3p::*;
use workspacer_interface::*;
use workspacer_crate::*;
use workspacer::*;

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {

    let workspace = Workspace::<PathBuf,CrateHandle>::new(&PathBuf::from(".")).await?;

    // 1) Assert Git working tree is clean
    workspace.ensure_git_clean().await?;

    // 2) Pin all wildcard dependencies to their Cargo.locked version
    workspace.pin_all_wildcard_dependencies().await?;

    println!("Successfully pinned wildcard dependencies!");

    Ok(())
}
