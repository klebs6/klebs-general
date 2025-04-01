// ---------------- [ File: workspacer-cli/src/write.rs ]
crate::ix!();

/// A single fused enum with subcommands, each having **all** the config flags duplicated.
/// This way, you can run `ws write single-crate ...` or `ws write workspace ...` etc.,
/// all from one top-level enum. No nested subcommand layering is required.
#[derive(StructOpt, Clone, Debug)]
pub enum ReadmeWriterCli {
    /// Generate or update README for a single crate
    SingleCrate {
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

        /// If set, the crate interface text is truncated if it exceeds this length.
        #[structopt(long = "max-interface-length")]
        max_interface_length: Option<usize>,

        /// Path to the crate directory containing Cargo.toml
        #[structopt(parse(from_os_str), long = "crate-path")]
        crate_path: PathBuf,

        /// If true, we write seeds (plant) but do not attempt to read expansions.
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },

    /// Generate or update READMEs for multiple distinct crates
    MultiCrate {
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

        /// If set, the crate interface text is truncated if it exceeds this length.
        #[structopt(long = "max-interface-length")]
        max_interface_length: Option<usize>,

        /// Paths to the crate directories containing Cargo.toml
        #[structopt(parse(from_os_str))]
        crate_paths: Vec<PathBuf>,

        /// If true, we write seeds (plant) but do not attempt to read expansions.
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },

    /// Generate or update READMEs for an entire workspace
    Workspace {
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

        /// If set, the crate interface text is truncated if it exceeds this length.
        #[structopt(long = "max-interface-length")]
        max_interface_length: Option<usize>,

        /// Path to the workspace directory containing Cargo.toml(s)
        #[structopt(parse(from_os_str), long = "workspace-path")]
        workspace_path: PathBuf,

        /// If true, we write seeds (plant) but do not attempt to read expansions.
        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing existing READMEs in the workspace
        #[structopt(long = "force")]
        force: bool,
    },
}

impl From<&ReadmeWriterCli> for ReadmeWriterConfig {
    fn from(cli: &ReadmeWriterCli) -> Self {
        trace!(
            "Converting &ReadmeWriterCli to ReadmeWriterConfig with skip_docs={}, skip_fn_bodies={}, include_test_items={}, include_private_items={}, max_interface_length={:?}",
            cli.skip_docs,
            cli.skip_fn_bodies,
            cli.include_test_items,
            cli.include_private_items,
            cli.max_interface_length
        );

        // If you have a builder, we can do:
        ReadmeWriterConfigBuilder::default()
            .skip_docs(cli.skip_docs)
            .skip_fn_bodies(cli.skip_fn_bodies)
            .include_test_items(cli.include_test_items)
            .include_private_items(cli.include_private_items)
            .max_interface_length(cli.max_interface_length)
            .build()
            .unwrap()
    }
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

impl ReadmeWriterCli {
    /// Runs the readme-writer logic for whichever subcommand was chosen.
    /// This merges the config flags directly into each variant, so each
    /// subcommand can be invoked with the same set of flags.
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            Self::SingleCrate {
                skip_docs,
                skip_fn_bodies,
                include_test_items,
                include_private_items,
                max_interface_length,
                crate_path,
                plant,
                force,
            } => {
                let config = ReadmeWriterConfigBuilder::default()
                    .skip_docs(*skip_docs)
                    .skip_fn_bodies(*skip_fn_bodies)
                    .include_test_items(*include_test_items)
                    .include_private_items(*include_private_items)
                    .max_interface_length(*max_interface_length)
                    .build()
                    .unwrap();

                trace!(
                    "readme-writer-cli: SingleCrate subcommand => path={:?}, plant={}, force={}, config={:?}",
                    crate_path, plant, force, config
                );

                let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&crate_path).await?));
                UpdateReadmeFiles::update_readme_files(handle, *plant, *force, &config).await?;
                info!("Done updating readme for single crate at {:?}", crate_path);
            }

            Self::MultiCrate {
                skip_docs,
                skip_fn_bodies,
                include_test_items,
                include_private_items,
                max_interface_length,
                crate_paths,
                plant,
                force,
            } => {
                let config = ReadmeWriterConfigBuilder::default()
                    .skip_docs(*skip_docs)
                    .skip_fn_bodies(*skip_fn_bodies)
                    .include_test_items(*include_test_items)
                    .include_private_items(*include_private_items)
                    .max_interface_length(*max_interface_length)
                    .build()
                    .unwrap();

                trace!(
                    "readme-writer-cli: MultiCrate subcommand => paths={:?}, plant={}, force={}, config={:?}",
                    crate_paths, plant, force, config
                );

                for path in crate_paths {
                    let path_buf = path.to_path_buf();
                    debug!("Processing crate at {:?}", path_buf);
                    let handle = Arc::new(AsyncMutex::new(CrateHandle::new(&path_buf).await?));
                    UpdateReadmeFiles::update_readme_files(handle, *plant, *force, &config).await?;
                    debug!("Finished readme updates for crate at {:?}", path_buf);
                }
                info!("Done updating readmes for all crates in MultiCrate mode.");
            }

            Self::Workspace {
                skip_docs,
                skip_fn_bodies,
                include_test_items,
                include_private_items,
                max_interface_length,
                workspace_path,
                plant,
                force,
            } => {
                let config = ReadmeWriterConfigBuilder::default()
                    .skip_docs(*skip_docs)
                    .skip_fn_bodies(*skip_fn_bodies)
                    .include_test_items(*include_test_items)
                    .include_private_items(*include_private_items)
                    .max_interface_length(*max_interface_length)
                    .build()
                    .unwrap();

                trace!(
                    "readme-writer-cli: Workspace subcommand => path={:?}, plant={}, force={}, config={:?}",
                    workspace_path, plant, force, config
                );

                let ws = Arc::new(AsyncMutex::new(
                    Workspace::<PathBuf, CrateHandle>::new(&workspace_path).await?
                ));
                UpdateReadmeFiles::update_readme_files(ws, *plant, *force, &config).await?;
                info!("Done updating readmes for entire workspace at {:?}", workspace_path);
            }
        }

        info!("All done with readme-writer-cli. Exiting successfully.");
        Ok(())
    }
}
