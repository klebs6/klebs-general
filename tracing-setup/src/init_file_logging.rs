crate::ix!();

pub struct FileLoggingConfiguration {
    pub log_path: Option<PathBuf>,
    pub log_level: Level,
    pub rotation: Option<Rotation>,
}

impl Default for FileLoggingConfiguration {
    fn default() -> Self {
        Self {
            log_path: Some(PathBuf::from("default.log")),
            log_level: Level::INFO,
            rotation: Some(Rotation::DAILY),
        }
    }
}

impl FileLoggingConfiguration {
    pub fn new(log_path: Option<PathBuf>, log_level: Level, rotation: Option<Rotation>) -> Self {
        Self {
            log_path,
            log_level,
            rotation,
        }
    }

    pub fn create_writer(&self) -> BoxMakeWriter {
        match &self.log_path {
            Some(log_path) => {
                if let Some(rotation) = &self.rotation {
                    let file_appender = RollingFileAppender::new(rotation.clone(), ".", log_path);
                    BoxMakeWriter::new(file_appender)
                } else {
                    let file = File::create(log_path).expect("Could not create log file");
                    BoxMakeWriter::new(Arc::new(file))
                }
            }
            None => BoxMakeWriter::new(io::stderr),
        }
    }
}

pub fn init_file_logging(config: FileLoggingConfiguration) {
    let writer = config.create_writer();

    let env_filter = std::env::var("LOGFLAG").unwrap_or_else(|_| "info".to_string());

    let subscriber = FmtSubscriber::builder()
        .with_max_level(config.log_level)
        .with_env_filter(EnvFilter::new(env_filter))
        .with_writer(writer)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    tracing::info!("Logging initialized with file rotation");
}

pub fn init_default_file_logging() {
    let config = FileLoggingConfiguration::default();
    init_file_logging(config);
}

#[cfg(test)]
mod file_logging_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tracing_subscriber::fmt::MakeWriter;

    #[test]
    fn test_create_writer_with_rotation() {
        let config = FileLoggingConfiguration::new(
            Some(PathBuf::from("test_rotation.log")),
            Level::DEBUG,
            Some(Rotation::DAILY),
        );
        let writer = config.create_writer();
        // Verify the writer is created successfully
        let _ = writer.make_writer();
        // Clean up
        let _ = fs::remove_file("test_rotation.log");
    }

    #[test]
    fn test_create_writer_without_rotation() {
        let config = FileLoggingConfiguration::new(
            Some(PathBuf::from("test_no_rotation.log")),
            Level::DEBUG,
            None,
        );
        let writer = config.create_writer();
        // Verify the writer is created successfully
        let _ = writer.make_writer();
        // Clean up
        let _ = fs::remove_file("test_no_rotation.log");
    }

    #[test]
    fn test_default_logging_configuration() {
        let config = FileLoggingConfiguration::default();
        assert_eq!(config.log_path.unwrap().to_str(), Some("default.log"));
        assert_eq!(config.log_level, Level::INFO);
        assert!(matches!(config.rotation, Some(Rotation::DAILY)));
    }

    #[test]
    fn test_init_file_logging_with_defaults() {
        init_default_file_logging();
        tracing::info!("This is a default log message.");
        tracing::debug!("This is a debug message, but won't appear with default level.");
        // Clean up
        let _ = fs::remove_file("default.log");
    }
}
