crate::ix!();

#[derive(Debug)]
pub struct CreateQueryBatches {
    pub path_to_workspace:   PathBuf,
    pub path_to_batchdir:    PathBuf,
}

impl CreateQueryBatches {

    pub async fn run(&self) -> Result<(),AIGenerateReadmesError> {

        let workspace = Workspace::new(&self.path_to_workspace).await?;

        for item in workspace.crates() {
            item.validate_integrity()?;
            println!("{:?}", item.as_ref());
        }

        workspace.validate_integrity()?;

        Ok(())
    }
}
