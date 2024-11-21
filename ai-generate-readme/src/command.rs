crate::ix!();

#[derive(Debug)]
pub enum AIGenerateReadmesCommand {
    CreateQueryBatches(CreateQueryBatches),
    SendBatchesToGpt {
        path_to_batchdir:          PathBuf,
        path_to_response_batchdir: PathBuf,
    },
    ParseGptResponses {
        path_to_response_batchdir: PathBuf,
    },
}

impl From<CreateQueryBatches> for AIGenerateReadmesCommand {

    fn from(x: CreateQueryBatches) -> AIGenerateReadmesCommand {
        AIGenerateReadmesCommand::CreateQueryBatches(x)
    }
}

impl AIGenerateReadmesCommand {

    pub async fn run(&self) -> Result<(),AIGenerateReadmesError> {

        match self {
            Self::CreateQueryBatches(item) => Ok(item.run().await?),

            Self::SendBatchesToGpt { path_to_batchdir, path_to_response_batchdir } => {
                todo!();
            }
            Self::ParseGptResponses { path_to_response_batchdir } => {
                todo!();
            }
        }
    }
}
