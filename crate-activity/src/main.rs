use crate_activity::*;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(),CrateActivityError> {

    let cli = CrateActivityCli::from_args();

    crate_activity_main(&cli).await?;

    Ok(())
}
