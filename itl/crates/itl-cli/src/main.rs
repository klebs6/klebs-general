#![forbid(unsafe_code)]

use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).init();

    tracing::info!("itl (L0): workspace wired; stubs compile");
    println!("itl: ok (L0)");
}
