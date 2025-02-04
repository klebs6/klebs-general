use world_city_and_street_db_builder::*;

// ---------------- [ File: src/main.rs ]
use tracing_setup::*;
use structopt::*;

#[tokio::main]
async fn main() -> Result<(),WorldCityAndStreetDbBuilderError> {
    configure_tracing();
    let cli = Cli::from_args();
    cli.run().await
}
