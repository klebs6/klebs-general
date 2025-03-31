// ---------------- [ File: workspacer-cli/src/meta.rs ]
crate::ix!();

/// 4) In your CLI code, define the `MetaSubcommand` with two variants:
#[derive(Debug, StructOpt)]
pub enum MetaSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl MetaSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            MetaSubcommand::Crate { crate_name } => {
                // a) Use `run_with_crate` or do it inline. 
                //    We'll demonstrate the `run_with_crate` approach:
                run_with_crate(crate_name.clone(), false, move |handle| {
                    Box::pin(async move {
                        // b) Now call your trait:
                        let metadata = handle.get_cargo_metadata().await.map_err(|crate_err| {
                            error!("Failed to retrieve metadata for crate='{}': {crate_err:?}", handle.name());
                            WorkspaceError::CrateError(crate_err)
                        })?;

                        // c) Print or process it:
                        println!("Cargo metadata for crate='{}':\n{:#?}", handle.name(), metadata);
                        Ok(())
                    })
                })
                .await
            }

            MetaSubcommand::Workspace { path } => {
                // a) Use `run_with_workspace`
                run_with_workspace(Some(path.clone()), false, move |ws| {
                    Box::pin(async move {
                        // b) call the trait
                        let metadata = ws.get_cargo_metadata().await.map_err(|we| {
                            error!("Failed to retrieve metadata for workspace: {we:?}");
                            we
                        })?;

                        // c) Print or process:
                        println!("Cargo metadata for workspace at '{}':\n{:#?}", ws.as_ref().display(), metadata);
                        Ok(())
                    })
                })
                .await
            }
        }
    }
}
