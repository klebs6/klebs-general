use crate_activity::*;

#[tokio::main]
async fn main() -> Result<(),CrateActivityError> {

    let cli = CrateActivityCli::read_command_line();

    crate_activity_main(&cli).await?;

    Ok(())
}
