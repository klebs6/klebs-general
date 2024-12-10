use crate_activity::*;

#[tokio::main]
async fn main() -> Result<(),CrateActivityError> {

    crate_activity_main().await?;

    Ok(())
}
