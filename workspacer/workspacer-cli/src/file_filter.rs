crate::ix!();

#[derive(StructOpt, Clone, Debug)]
pub enum FileFilterCli {
    /// Filter/transform text in multiple files, given:
    ///   1) A 'file list' text file, where each line is a path to a file
    ///   2) A 'instructions' file, containing instructions for how to transform the content
    /// We also do a git-clean check unless `--skip-git-check` is set. 
    /// Overwrites original files if successful.
    Files {
        /// Path to either a single crate or a workspace (if omitted, uses current dir)
        #[structopt(long)]
        path: Option<PathBuf>,

        /// Path to a text file with one file path per line
        #[structopt(long)]
        file_list: PathBuf,

        /// Path to a file containing user instructions
        #[structopt(long)]
        instructions_file: PathBuf,

        /// If set, we will store brand-new queries to the AI for unprocessed items
        /// and then exit, waiting for the user to plant seeds. Defaults to false.
        #[structopt(long)]
        plant: bool,

        /// If set, skip the Git cleanliness check.
        #[structopt(long)]
        skip_git_check: bool,

        /// Optional max file size in bytes (default ~512k).
        #[structopt(long)]
        max_file_size_bytes: Option<u64>,
    },
}

impl FileFilterCli {

    #[tracing::instrument(level="trace")]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            Self::Files {
                path,
                file_list,
                instructions_file,
                plant,
                skip_git_check,
                max_file_size_bytes,
            } => {
                // 1) Detect single crate vs. workspace
                let detect_path = path.as_deref().unwrap_or_else(|| {
                    // fallback to current dir if none provided
                    std::path::Path::new(".")
                });
                let mut single_or_ws = SingleOrWorkspace::detect(detect_path).await?;

                // 2) Possibly ensure Git is clean
                if !skip_git_check {
                    single_or_ws.ensure_git_clean().await?;
                } else {
                    debug!("Skipping Git cleanliness check because --skip-git-check was provided.");
                }

                // 3) Build our config
                let cfg = FileFilterConfigBuilder::default()
                    .max_file_size_bytes(*max_file_size_bytes)
                    .build()
                    .unwrap();

                trace!("Built FileFilterConfig => {:?}", cfg);

                // 4) Read instructions
                let instructions_content = tokio::fs::read_to_string(&instructions_file).await.map_err(|io_err| {
                    error!("Failed to read instructions from '{}': {:?}", instructions_file.display(), io_err);
                    WorkspaceError::IoError {
                        io_error: Arc::new(io_err),
                        context: format!("reading instructions file at {}", instructions_file.display()),
                    }
                })?;

                // 5) Apply the text filter
                //    We'll call `apply_text_filter_to_files(...)`, which returns
                //    an `Result<(), AiFileFilterError>`. So we need to map that error
                //    into a `WorkspaceError`.
                info!("Filtering text for files listed in {}", file_list.display());

                apply_text_filter_to_files(
                    file_list,
                    &instructions_content,
                    *plant,
                    &cfg,
                )
                .await
                .map_err(map_filter_error_into_workspace_error)?;

                // 6) Validate integrity on single crate or workspace
                info!("Validating integrity after file-filtering operation...");
                single_or_ws.validate_integrity().await?;

                info!("All done with file filter subcommand.");
            }
        }

        Ok(())
    }
}
