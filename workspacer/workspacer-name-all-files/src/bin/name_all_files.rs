// ---------------- [ File: src/bin/name_all_files.rs ]
//
// A binary entrypoint that attempts to name all files in either a Rust workspace
// or a single crate, depending on the current directory’s Cargo.toml.
use workspacer_3p::*;
use workspacer_errors::*;
use workspacer_crate::*;
use workspacer_workspace::*;
use workspacer_name_all_files::*;

#[tokio::main]
async fn main() -> Result<(),WorkspaceError> {

    configure_tracing();

    let path = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            error!("Could not get current directory: {}", e);
            std::process::exit(1);
        }
    };

    match Workspace::<PathBuf,CrateHandle>::new(&path).await {
        Ok(ws) => {
            if let Err(e) = ws.name_all_files().await {
                error!("Error naming all files in workspace:\n{}", e);
                std::process::exit(1);
            }
            info!("Successfully named all files for the workspace.");
        }
        Err(WorkspaceError::ActuallyInSingleCrate { .. }) => {
            // Not a real workspace; treat it as a single crate
            match CrateHandle::new(&path).await {
                Ok(ch) => {
                    if let Err(e) = ch.name_all_files().await {
                        error!("Error naming all files in single crate:\n{:?}", e);
                        std::process::exit(1);
                    }
                    info!("Successfully named all files for the single crate.");
                }
                Err(e) => {
                    error!("Could not open crate or workspace: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(other) => {
            error!("Could not open workspace: {}", other);
            std::process::exit(1);
        }
    }

    Ok(())
}
