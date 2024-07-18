crate::ix!();

use system::*;

pub fn init_file_logging() {

    let logpath = std::env::var("LOGFILE").ok();
    let logflag = std::env::var("LOGFLAG").ok();

    let msg = format!{
        "in function init_file_logging with LOGFILE={} and LOGFLAG={}",
        logpath.as_ref().unwrap(),
        logflag.as_ref().unwrap(),
    };

    let file = match logpath {
        Some(p) => match File::create(p) {
            Ok(f)  => Some(f),
            Err(_e) => None,
        },
        None    => None,
    };

    let writer = match file {
        Some(file) => BoxMakeWriter::new(Arc::new(file)),
        None       => BoxMakeWriter::new(io::stderr),
    };

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher
        // than TRACE (e.g, debug, info, warn,
        // etc.) will be written to stdout.
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_env("LOGFLAG"))
        .json()
        .with_writer(writer)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    tracing::info!("test trace info from get-me-tables!");
}

/*
pub fn alt_setup_logging() {

    // Configure a custom event formatter
    let format = fmt::format()
        .json()
        .with_level(false)       // don't include levels in formatted output
        .with_target(false)      // don't include targets
        .with_thread_ids(true)   // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .compact();              // use the `Compact` formatting style.

    // Create a `fmt` subscriber that uses our
    // custom event format, and set it as the
    // default.
    tracing_subscriber::fmt()
        .event_format(format)
        .init();
}
*/
