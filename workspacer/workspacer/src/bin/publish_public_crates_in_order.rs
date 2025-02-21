// ---------------- [ File: src/bin/publish_public_crates_in_order.rs ]
//! This binary replicates (in Rust) the logic of a shell script that
//! publishes all public crates in a workspace to crates.io, in
//! topological order, skipping crates that are already published.

use structopt::StructOpt;
use workspacer::*;
use workspacer_interface::*;
use workspacer_crate::*;
use workspacer_3p::*;
use workspacer_consolidate::*; // only if needed

/// Command line interface for publishing public crates in topological order.
#[derive(StructOpt, Debug)]
#[structopt(name = "workspacer-publish-public-crates-in-order")]
pub struct PublishPublicCratesInOrderCli {
    /// Path to the workspace root
    #[structopt(long)]
    workspace_path: Option<PathBuf>,
}

impl PublishPublicCratesInOrderCli {
    /// Entry point for running the publish process.
    pub async fn run_publish_public_crates_in_order_cli(&self) -> Result<(), WorkspaceError> {
        // 1) Identify workspace path
        let path = self
            .workspace_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));

        // 2) Construct the workspace
        //    We'll rely on the existing `Workspace::new` method (AsyncTryFrom).
        let workspace = Workspace::<PathBuf, CrateHandle>::new(&path).await?;

        // 3) Publish the crates in topological order
        workspace.publish_public_crates_in_topological_order().await
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();
    let cli = PublishPublicCratesInOrderCli::from_args();
    cli.run_publish_public_crates_in_order_cli().await
}
