use rck::*;
use clap::{Arg, Command};
use tracing::*;

fn main() -> Result<(), RckError> {
    let matches = Command::new("rck")
        .version("1.0")
        .author("Author <email@example.com>")
        .about("Repo Check Tool")
        .arg(
            Arg::new("manifest")
                .short('m')
                .long("manifest")
                .value_name("FILE")
                .help("Sets a custom manifest file")
                .global(true)  // <-- This makes the --manifest argument global
                .action(clap::ArgAction::Set),
        )
        .subcommand(Command::new("status").about("Check status of repositories"))
        .subcommand(Command::new("sync").about("Sync repositories"))
        .get_matches();

    let default_manifest_path = DEFAULT_MANIFEST_PATH.to_string();
    let manifest_path = matches.get_one::<String>("manifest").unwrap_or(&default_manifest_path);
    let manifest = Manifest::load(manifest_path).expect("Failed to load manifest");

    if let Some(_) = matches.subcommand_matches("status") {
        manifest.process_repos(RepoOperation::Status)?;
    } else if let Some(_) = matches.subcommand_matches("sync") {
        manifest.process_repos(RepoOperation::Sync)?;
    } else {
        error!("No valid command provided.");
        std::process::exit(1);
    }

    Ok(())
}
