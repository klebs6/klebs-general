// ---------------- [ File: workspacer-cli/src/write.rs ]
crate::ix!();

/// A single fused enum with subcommands, each having **all** the config flags duplicated.
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

impl ReadmeWriterCli {
    /// Runs the readme-writer logic for whichever subcommand was chosen.
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
                // Build config from these fields:
                let config = ReadmeWriterConfigBuilder::default()
                    .skip_docs(*skip_docs)
                    .skip_fn_bodies(*skip_fn_bodies)
                    .include_test_items(*include_test_items)
                    .include_private_items(*include_private_items)
                    .max_interface_length(*max_interface_length)
                    .build()
                    .unwrap();

                trace!(
                    "SingleCrate => path={:?}, plant={}, force={}, config={:?}",
                    crate_path, plant, force, config
                );

                let handle = Arc::new(AsyncMutex::new(
                    CrateHandle::new(crate_path).await?
                ));
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
                    "MultiCrate => paths={:?}, plant={}, force={}, config={:?}",
                    crate_paths, plant, force, config
                );

                for path in crate_paths {
                    let handle = Arc::new(AsyncMutex::new(
                        CrateHandle::new(path).await?
                    ));
                    UpdateReadmeFiles::update_readme_files(handle, *plant, *force, &config).await?;
                    debug!("Finished readme updates for crate at {:?}", path);
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
                    "Workspace => path={:?}, plant={}, force={}, config={:?}",
                    workspace_path, plant, force, config
                );

                let ws = Arc::new(AsyncMutex::new(
                    Workspace::<PathBuf, CrateHandle>::new(workspace_path).await?
                ));
                UpdateReadmeFiles::update_readme_files(ws, *plant, *force, &config).await?;
                info!("Done updating readmes for entire workspace at {:?}", workspace_path);
            }
        }

        info!("All done with readme-writer-cli. Exiting successfully.");
        Ok(())
    }
}
