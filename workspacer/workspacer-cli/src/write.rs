// ---------------- [ File: workspacer-cli/src/write.rs ]
crate::ix!();

/// Command-line interface for our "readme-writer" binary.
/// It can operate on:
/// 1) A single crate path
/// 2) Multiple crate paths
/// 3) A workspace path
#[derive(StructOpt,Clone,Debug)]
pub struct ReadmeWriterCli {
    /// If set, do not include doc comments in the crate interface.
    #[structopt(long = "skip-docs")]
    skip_docs: bool,

    /// If set, do not include function bodies in the crate interface.
    #[structopt(long = "skip-fn-bodies")]
    skip_fn_bodies: bool,

    /// If set, include test items (`#[cfg(test)]`) in the crate interface.
    #[structopt(long = "include-test-items")]
    include_test_items: bool,

    /// If set, include private (non-pub) items in the crate interface.
    #[structopt(long = "include-private-items")]
    include_private_items: bool,

    /// If set, the crate interface text is truncated by removing doc comments and function
    /// bodies if it exceeds this threshold. 
    /// Otherwise, we attempt to gather the full interface.
    #[structopt(long = "max-interface-length")]
    max_interface_length: Option<usize>,

    #[structopt(subcommand)]
    command: ReadmeWriterCommand,
}

impl Into<ReadmeWriterConfig> for ReadmeWriterCli {
    fn into(self) -> ReadmeWriterConfig {
        trace!(
            "Converting ReadmeWriterCli to ReadmeWriterConfig with: skip_docs={}, skip_fn_bodies={}, include_test_items={}, include_private_items={}, max_interface_length={:?}",
            self.skip_docs,
            self.skip_fn_bodies,
            self.include_test_items,
            self.include_private_items,
            self.max_interface_length
        );

        ReadmeWriterConfigBuilder::default()
            .skip_docs(self.skip_docs)
            .skip_fn_bodies(self.skip_fn_bodies)
            .include_test_items(self.include_test_items)
            .include_private_items(self.include_private_items)
            .max_interface_length(self.max_interface_length)
            .build()
            .unwrap()
    }
}

/// Subcommands for different ways of using the tool
#[derive(StructOpt,Clone,Debug)]
pub enum ReadmeWriterCommand {
    /// Run on a single crate path
    SingleCrate {
        /// Path to the crate directory containing Cargo.toml
        #[structopt(parse(from_os_str))]
        crate_path: PathBuf,

        /// If true, we write seeds to the filesystem and do not attempt to read expansions yet
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },
    /// Run on multiple crate paths
    MultiCrate {
        /// Paths to crate directories
        #[structopt(parse(from_os_str))]
        crate_paths: Vec<PathBuf>,

        /// If true, we write seeds to the filesystem and do not attempt to read expansions yet
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },
    /// Run on a full workspace directory
    Workspace {
        /// Path to the workspace directory containing Cargo.toml(s)
        #[structopt(parse(from_os_str))]
        workspace_path: PathBuf,

        /// If true, we write seeds to the filesystem and do not attempt to read expansions yet
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },
}

impl ReadmeWriterCli {

    pub async fn run(&self) -> Result<(), WorkspaceError> {

        let config: ReadmeWriterConfig = self.into();

        trace!("Parsed CLI arguments: {:?}", self);

        match self.command {
            ReadmeWriterCommand::SingleCrate { crate_path, plant, force } => {
                info!("readme-writer-cli: SingleCrate mode selected for path = {:?}", crate_path);

                let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&crate_path).await?));
                info!("Starting readme updates for single crate at {:?}", crate_path);

                UpdateReadmeFiles::update_readme_files(
                    handle.clone(),
                    plant,
                    force,
                    &config
                ).await?;

                info!("Done updating readme for single crate at {:?}", crate_path);
            }
            ReadmeWriterCommand::MultiCrate { crate_paths, plant, force } => {
                info!("readme-writer-cli: MultiCrate mode for paths = {:?}", crate_paths);

                for path in crate_paths {
                    debug!("Processing crate at {:?}", path);
                    let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&path).await?));

                    UpdateReadmeFiles::update_readme_files(
                        handle.clone(),
                        plant,
                        force,
                        &config
                    ).await?;

                    debug!("Finished readme updates for crate at {:?}", path);
                }
                info!("Done updating readmes for all crates in MultiCrate mode.");
            }
            ReadmeWriterCommand::Workspace { workspace_path, plant, force } => {
                info!("readme-writer-cli: Workspace mode selected for path = {:?}", workspace_path);

                let ws = Arc::new(AsyncMutex::new(
                        Workspace::<PathBuf, CrateHandle>::new(&workspace_path).await?
                ));
                info!("Starting readme updates for full workspace at {:?}", workspace_path);

                UpdateReadmeFiles::update_readme_files(
                    ws.clone(),
                    plant,
                    force,
                    &config
                ).await?;

                info!("Done updating readmes for workspace at {:?}", workspace_path);
            }
        }

        info!("All done with readme-writer-cli. Exiting successfully.");
        Ok(())
    }
}
