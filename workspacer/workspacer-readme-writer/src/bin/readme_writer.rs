// ---------------- [ File: workspacer-readme-writer/src/bin/readme_writer.rs ]
use workspacer_3p::*;
use structopt::*;
use workspacer_readme_writer::*;
use workspacer_workspace::*;
use workspacer_crate::*;

/// Command-line interface for our "readme-writer" binary.
/// It can operate on:
/// 1) A single crate path
/// 2) Multiple crate paths
/// 3) A workspace path
#[derive(StructOpt, Debug)]
#[structopt(name = "readme-writer", about = "CLI for AI-based README & Cargo.toml updates")]
struct ReadmeWriterCli {
    #[structopt(subcommand)]
    command: ReadmeWriterCommand,
}

/// Subcommands for different ways of using the tool
#[derive(StructOpt, Debug)]
enum ReadmeWriterCommand {
    /// Run on a single crate path
    SingleCrate {
        /// Path to the crate directory containing Cargo.toml
        #[structopt(parse(from_os_str))]
        crate_path: std::path::PathBuf,
    },
    /// Run on multiple crate paths
    MultiCrate {
        /// Paths to crate directories
        #[structopt(parse(from_os_str))]
        crate_paths: Vec<std::path::PathBuf>,
    },
    /// Run on a full workspace directory
    Workspace {
        /// Path to the workspace directory containing Cargo.toml(s)
        #[structopt(parse(from_os_str))]
        workspace_path: std::path::PathBuf,
    },
}


#[tokio::main]
pub async fn main() -> Result<(), AiReadmeWriterError> {
    configure_tracing();

    // Parse the CLI arguments
    let args = ReadmeWriterCli::from_args();
    trace!("Parsed CLI arguments: {:?}", args);

    match args.command {
        ReadmeWriterCommand::SingleCrate { crate_path } => {
            info!("readme-writer-cli: SingleCrate mode selected for path = {:?}", crate_path);

            // 1) Build a CrateHandle from the given path
            //    (using your real creation method).
            // 2) Then call `update_readme_files` on it.

            let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&crate_path).await?));
            info!("Starting readme updates for single crate at {:?}", crate_path);

            // The trait method from `UpdateReadmeFiles`
            UpdateReadmeFiles::update_readme_files(handle.clone()).await?;

            info!("Done updating readme for single crate at {:?}", crate_path);
        }
        ReadmeWriterCommand::MultiCrate { crate_paths } => {
            info!("readme-writer-cli: MultiCrate mode for paths = {:?}", crate_paths);

            // For each path, do basically what we did above
            for path in crate_paths {
                debug!("Processing crate at {:?}", path);

                let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&path).await?));

                UpdateReadmeFiles::update_readme_files(handle.clone()).await?;

                debug!("Finished readme updates for crate at {:?}", path);
            }
            info!("Done updating readmes for all crates in MultiCrate mode.");
        }
        ReadmeWriterCommand::Workspace { workspace_path } => {
            info!("readme-writer-cli: Workspace mode selected for path = {:?}", workspace_path);

            // 1) Build a Workspace object (depending on your real code).
            // 2) Then call `update_readme_files` on it.

            let ws = Arc::new(AsyncMutex::new(Workspace::<std::path::PathBuf, CrateHandle>::new(&workspace_path).await?));

            info!("Starting readme updates for full workspace at {:?}", workspace_path);
            UpdateReadmeFiles::update_readme_files(ws.clone()).await?;

            info!("Done updating readmes for workspace at {:?}", workspace_path);
        }
    }

    info!("All done with readme-writer-cli. Exiting successfully.");
    Ok(())
}
