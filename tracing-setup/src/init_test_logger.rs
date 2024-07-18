crate::ix!();

pub fn init_test_logger(
    level:                  LevelFilter, 
    limit_to_current_crate: bool
) {

    let mut builder = env_logger::builder();

    builder.filter_level(level);

    if limit_to_current_crate {

        let crate_name = env!("CARGO_PKG_NAME");

        // Ensure only logs from the current crate are captured
        builder.filter_module(crate_name, level);
    }

    // Ensure events are captured by `cargo test`
    //
    builder.is_test(true);

    // Ignore errors initializing the logger if tests race to configure it
    //
    builder.try_init().expect("could not initialize test logger")
}

pub fn init_test_logger_with_max_level_filter() {

    // Include all events in tests
    let level = LevelFilter::max();

    let _ = env_logger::builder()
        .filter_level(level)
        // Ensure events are captured by `cargo test`
        .is_test(true)
        // Ignore errors initializing the logger if tests race to configure it
        .try_init();
}


#[macro_export] macro_rules! setup_test_logger {
    () => {
        init_test_logger(LevelFilter::Info, true);
        //init_test_logger_old();
    }
}
