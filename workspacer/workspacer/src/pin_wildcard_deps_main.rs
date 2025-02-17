//! src/bin/pin_wildcards.rs
use workspacer_3p::*;
use workspacer_interface::*;
use workspacer_crate::*;
use workspacer::*;

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();
    
    let path = PathBuf::from(".");

    // 1) Try creating a workspace
    match Workspace::<PathBuf,CrateHandle>::new(&path).await {
        Ok(workspace) => {
            // It's a valid workspace
            workspace.ensure_git_clean().await?; 
            workspace.pin_all_wildcard_dependencies().await?;
            println!("Successfully pinned wildcard dependencies in the workspace!");
        }
        Err(WorkspaceError::InvalidWorkspace { invalid_workspace_path: _ }) => {
            // 2) Not a valid workspace -> fallback to single crate
            println!("Not a valid workspace; falling back to single crate logic...");
            
            let single = CrateHandle::new(&path).await?;
            single.ensure_git_clean().await?;
            single.pin_all_wildcard_dependencies().await?;
            println!("Successfully pinned wildcard dependencies in the single crate!");
        }
        Err(e) => {
            // Some other workspace error
            eprintln!("Workspace creation failed with error: {:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
