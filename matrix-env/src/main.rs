#![forbid(unsafe_code)]

use matrix_env::Layer0Entrypoint;

fn main() -> std::process::ExitCode {
    Layer0Entrypoint::run_from_env()
}
