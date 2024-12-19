use usa_city_and_street_db_builder::*;
use tracing_setup::*;
use structopt::*;

#[tokio::main]
async fn main() -> Result<(),UsaCityAndStreetDbBuilderError> {
    configure_tracing();

    let cli = Cli::from_args();
    cli.run().await
}
