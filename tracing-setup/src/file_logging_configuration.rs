crate::ix!();

pub struct FileLoggingConfiguration {
    log_path:  Option<PathBuf>,
    log_level: Level,
    rotation:  Option<Rotation>,
    temporary: bool,
}

impl FileLoggingConfiguration {

    pub fn default_temporary() -> Self {
        Self {
            log_path:  Some(PathBuf::from("default.log")),
            log_level: Level::INFO,
            rotation:  Some(Rotation::DAILY),
            temporary: true,
        }
    }

    pub fn new(log_path: Option<PathBuf>, log_level: Level, rotation: Option<Rotation>) -> Self {
        Self {
            log_path,
            log_level,
            rotation,
            temporary: false,
        }
    }

    pub fn new_temporary(
        log_path:  Option<PathBuf>, 
        log_level: Level, 
        rotation:  Option<Rotation>

    ) -> Self {

        Self {
            log_path,
            log_level,
            rotation,
            temporary: true,
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

    //----------------------------------------[access]
    pub fn log_level(&self) -> &Level {
        &self.log_level
    }

    pub fn log_path(&self) -> &Option<PathBuf> {
        &self.log_path
    }

    pub fn rotation(&self) -> &Option<Rotation> {
        &self.rotation
    }

    pub fn info_level(&self) -> bool {
        self.log_level == Level::INFO
    }

    pub fn log_path_root_is(&self, path: impl AsRef<Path>) -> bool {
        self.log_path == Some(path.as_ref().to_path_buf())
    }

    pub fn rotates_daily(&self) -> bool {
        matches!(self.rotation(), Some(Rotation::DAILY))
    }

    pub fn remove_logs(&self) {
        if let Some(path) = &self.log_path {
            match std::fs::remove_file(path) {
                Ok(()) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // File does not exist, no problem
                },
                Err(e) => {
                    panic!("Failed to remove log file: {:?}", e);
                }
            }
        }
    }

    pub fn is_temporary(&self) -> bool {
        self.temporary
    }
}

impl Default for FileLoggingConfiguration {

    fn default() -> Self {
        Self {
            log_path: Some(PathBuf::from("default.log")),
            log_level: Level::INFO,
            rotation: Some(Rotation::DAILY),
            temporary: false,
        }
    }
}

impl Drop for FileLoggingConfiguration {

    fn drop(&mut self) {
        if self.is_temporary() {
            self.remove_logs();
        }
    }
}
