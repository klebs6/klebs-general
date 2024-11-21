use ai_generate_the_readmes::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(),AIGenerateReadmesError> {

    let command = AIGenerateReadmesCommand::from(CreateQueryBatches { 
        path_to_workspace: PathBuf::from("/Users/kleb/bethesda/work/repo/aloe-rs"),
        path_to_batchdir:  PathBuf::from("./readme-generation-queries"),
    });

    println!("running command: {:#?}", command);

    command.run().await?;

    Ok(())
}
