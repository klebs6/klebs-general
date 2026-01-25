// ---------------- [ File: batch-mode-tts/src/bin/main.rs ]
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, error};

use batch_mode_tts::*;

/// ---------------------------------------------------------------------------
/// Command‑line interface for one‑shot, batch‑mode text‑to‑speech.
/// ---------------------------------------------------------------------------
#[derive(StructOpt, Debug)]
#[structopt(
    name    = "batch‑mode‑tts",
    about   = "Convert TEXT → SPEECH via OpenAI in an offline batch",
    author  = "kleb <tpk3.mx@gmail.com>",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    /// UTF‑8 text file to read.
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output audio file (e.g. out.mp3).
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

/// Build the job and execute it.
async fn execute_batch_mode_tts(cli: Cli) -> Result<(), BatchModeTtsError> {
    let job = BatchModeTtsJobBuilder::default()
        .input_path(cli.input)
        .output_path(cli.output)
        .build()?;

    job.run().await
}

/// Entrypoint.
#[tokio::main]
async fn main() {
    tracing_setup::configure_tracing();

    let cli = Cli::from_args();
    info!(
        "batch‑mode‑tts starting (input={}, output={})",
        cli.input.display(),
        cli.output.display()
    );

    if let Err(e) = execute_batch_mode_tts(cli).await {
        error!("batch‑mode‑tts failed: {e}");
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    info!("batch‑mode‑tts completed successfully");
}
