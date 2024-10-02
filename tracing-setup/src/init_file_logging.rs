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
        use chrono::Local;
        use std::fs;

        let config = FileLoggingConfiguration::default();
        let subscriber = create_file_logging_subscriber(&config);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("This is a default log message.");
            tracing::debug!("This is a debug message, but won't appear with default level.");

            // Wait briefly to ensure the log is written
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Determine the log file name with the date suffix
            let date_suffix = Local::now().format("%Y-%m-%d").to_string();
            let log_file_name = format!("default.log.{}", date_suffix);

            // Read the log file
            let log_contents = fs::read_to_string(&log_file_name).expect("Failed to read log file");
            assert!(log_contents.contains("This is a default log message."));
            assert!(!log_contents.contains("This is a debug message"));

            // Clean up
            let _ = fs::remove_file(log_file_name);
        });
    }
}
