#![allow(unused_imports)]
#[macro_use]
mod imports;
use imports::*;

// NOTE: this public export is useful for traced-test
pub use colored;

x! {buffered_layer}
x! {configure_tracing}
x! {flushable}
x! {init_test_logger}
x! {init_file_logging}
x! {dynamic_level}
x! {file_logging_configuration}
x! {log_level}
x! {event_printer}
x! {setup_and_buffered_subscriber}

/// Runtime support symbols used by `traced-test` procedural macro expansions.
///
/// This module gives generated test code one stable dependency boundary instead
/// of requiring each consumer crate to import `colored`, `Flushable`,
/// `EventPrinter`, or tracing setup functions manually.
pub mod traced_test_support {
    pub use colored::Colorize;

    pub use crate::event_printer::EventPrinter;
    pub use crate::flushable::Flushable;
    pub use crate::setup_and_buffered_subscriber::{
        setup_buffered_tracing, setup_default_buffered_tracing,
    };
}
