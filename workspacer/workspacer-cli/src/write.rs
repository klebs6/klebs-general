// ---------------- [ File: workspacer-cli/src/write.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum WriteSubcommand {
    /// Write or update the README for a single crate
    CrateReadme {
        #[structopt(long = "crate")]
        crate_name: PathBuf,

        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },

    /// Write or update the README for all crates in the workspace
    AllReadmes {
        #[structopt(long = "workspace")]
        workspace_path: PathBuf,

        #[structopt(long = "plant")]
        plant: bool,

        /// Pass this to force re-writing an existing README
        #[structopt(long = "force")]
        force: bool,
    },
}

impl WriteSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            WriteSubcommand::CrateReadme { crate_name, plant, force } => {
                // build crate handle
                let handle = CrateHandle::new(crate_name)
                    .await
                    .map_err(WorkspaceError::CrateError)?;
                let handle_arc = Arc::new(AsyncMutex::new(handle));

                // Now call update_readme_files
                UpdateReadmeFiles::update_readme_files(handle_arc, *plant, *force)
                    .await
                    .map_err(|ai_err: AiReadmeWriterError| {
                        error!("AiReadmeWriterError={:#?}",ai_err);
                        WorkspaceError::ReadmeWriteError(ReadmeWriteError::AiReadmeWriterError)
                    })?;
                Ok(())
            }

            WriteSubcommand::AllReadmes { workspace_path, plant, force } => {
                // build workspace
                let ws = Workspace::<PathBuf, CrateHandle>::new(workspace_path).await?;

                let ws_arc = Arc::new(AsyncMutex::new(ws));
                UpdateReadmeFiles::update_readme_files(ws_arc, *plant, *force)
                    .await
                    .map_err(|ai_err: AiReadmeWriterError| {
                        error!("AiReadmeWriterError={:#?}",ai_err);
                        WorkspaceError::ReadmeWriteError(ReadmeWriteError::AiReadmeWriterError)
                    })?;
                Ok(())
            }
        }
    }
}
