crate::ix!();

pub fn init_default_file_logging() {
    let config = FileLoggingConfiguration::default();
    init_file_logging(config);
}

pub fn create_file_logging_subscriber(config: &FileLoggingConfiguration) -> impl Subscriber + Send + Sync {
    let writer = config.create_writer();

    let env_filter = std::env::var("LOGFLAG").unwrap_or_else(|_| "info".to_string());

    FmtSubscriber::builder()
        .with_max_level(*config.log_level())
        .with_env_filter(EnvFilter::new(env_filter))
        .with_writer(writer)
        .finish()
}

pub fn init_file_logging(config: FileLoggingConfiguration) {
    static INIT: std::sync::Once = std::sync::Once::new();
    static GUARD: Mutex<Option<tracing::subscriber::DefaultGuard>> = Mutex::new(None);

    INIT.call_once(|| {
        // Create the subscriber
        let subscriber = create_file_logging_subscriber(&config);

        if tracing::subscriber::set_global_default(subscriber).is_err() {
            eprintln!("Global subscriber already set, proceeding without setting it again.");

            // Re-create the subscriber since the previous one was moved
            let subscriber = create_file_logging_subscriber(&config);

            // Set the subscriber for the current thread
            let guard = tracing::subscriber::set_default(subscriber);
            *GUARD.lock().unwrap() = Some(guard);
            // The guard is stored in a Mutex to keep it alive
        } else {
            tracing::info!("Logging initialized with file rotation");
        }
    });
}

#[cfg(test)]
mod file_logging_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tracing_subscriber::fmt::MakeWriter;

    #[test]
    #[serial]
    fn test_create_writer_with_rotation() {
        let log_file = PathBuf::from("test_rotation.log");
        let config = FileLoggingConfiguration::new_temporary(
            Some(log_file.clone()),
            Level::DEBUG,
            Some(Rotation::DAILY),
        );

        let subscriber = create_file_logging_subscriber(&config);

        tracing::subscriber::with_default(subscriber, || {
            // Log some messages
            tracing::debug!("This is a test debug message");
        });

        // Clean up
        config.remove_logs();
    }

    #[test]
    #[serial]
    fn test_create_writer_without_rotation() {
        let log_file = PathBuf::from("test_no_rotation.log");
        let config = FileLoggingConfiguration::new_temporary(
            Some(log_file.clone()),
            Level::DEBUG,
            None,
        );
        let writer = config.create_writer();
        // Verify the writer is created successfully
        let _ = writer.make_writer();

        // Clean up
        config.remove_logs();
    }

    #[test]
    #[serial]
    fn test_default_logging_configuration() {
        let config = FileLoggingConfiguration::default_temporary();
        assert!(config.log_path_root_is("default.log"));
        assert!(config.info_level());
        assert!(config.rotates_daily());
    }

    #[test]
    #[serial]
    fn test_init_file_logging_with_defaults() {
        use tracing::{trace, debug, info, warn, error};

        info!("Starting test_init_file_logging_with_defaults");

        let config = FileLoggingConfiguration::default();
        let subscriber = create_file_logging_subscriber(&config);

        tracing::subscriber::with_default(subscriber, || {
            info!("This is a default log message.");
            debug!("This debug message should not appear at INFO level.");

            // Give the logging system a brief moment to write the file.
            trace!("sleeping briefly to allow log writes to occur");
            std::thread::sleep(std::time::Duration::from_millis(100));

            info!("Searching for any log file that starts with 'default.log'");

            // Because daily rotation may append date/time or other suffixes, we just look for any file
            // whose name begins with "default.log" in the current directory.
            let pattern = "default.log";
            let log_files: Vec<String> = std::fs::read_dir(".")
                .expect("Could not read current directory")
                .filter_map(|entry| {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(e) => {
                            warn!("Skipping DirEntry read error: {:?}", e);
                            return None;
                        }
                    };
                    let filename = entry.file_name().into_string().ok()?;
                    if filename.starts_with(pattern) {
                        Some(filename)
                    } else {
                        None
                    }
                })
                .collect();

            info!("Found candidate log files: {:?}", log_files);
            assert!(
                !log_files.is_empty(),
                "Expected at least one file matching 'default.log*', but found none"
            );

            // Check each candidate file for our INFO log message.
            let mut found_info_message = false;
            for file in &log_files {
                trace!("Checking file: {}", file);

                let log_contents = match std::fs::read_to_string(file) {
                    Ok(contents) => contents,
                    Err(e) => {
                        warn!("Failed to read file {}: {:?}", file, e);
                        continue;
                    }
                };

                if log_contents.contains("This is a default log message.") {
                    found_info_message = true;
                    break;
                }
            }

            assert!(
                found_info_message,
                "Did not find the expected 'default log message' in any candidate file"
            );

            // Cleanup: remove any files we created (especially if daily rotation introduced many).
            trace!("Removing created files");
            for file in &log_files {
                let _ = std::fs::remove_file(file);
            }
        });

        info!("test_init_file_logging_with_defaults completed successfully");
    }

}
