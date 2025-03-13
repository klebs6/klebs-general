// ---------------- [ File: workspacer-config/src/directory.rs ]
crate::ix!();

/// Represents the path to a `.ws` directory (local or global), along with
/// convenience methods for creating subdirectories or referencing the
/// `workspacer-config` file inside it.
///
/// This struct *differentiates* the notion of "the `.ws` directory" from "the config file."
/// It also allows you to do things like:
///  - Create `.ws/readme-writer-workspace`
///  - Create `.ws/test-upgrader-workspace`
///  - Clear or remove those subdirectories, etc.
#[derive(Debug, Clone)]
pub struct WorkspacerDir {
    root: PathBuf,
}

impl WorkspacerDir {
    /// Creates a `WorkspacerDir` pointing to the local `.ws` directory in
    /// the current working directory (e.g., `./.ws`).
    pub fn local() -> Self {
        Self {
            root: PathBuf::from(".ws"),
        }
    }

    /// Creates a `WorkspacerDir` pointing to the global `.ws` directory in
    /// the user's home folder (e.g., `~/.ws`).
    ///
    /// Returns an error if we cannot locate the home directory.
    pub fn global() -> Result<Self, WorkspacerFallbackError> {
        if let Some(home) = dirs::home_dir() {
            Ok(Self {
                root: home.join(".ws"),
            })
        } else {
            warn!("Could not locate home directory while constructing global WorkspacerDir.");
            Err(WorkspacerFallbackError::NoHomeDirectory)
        }
    }

    /// Returns the full path to this `.ws` directory.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Returns the file path for `workspacer-config` inside this `.ws` directory.
    pub fn config_file_path(&self) -> PathBuf {
        self.root.join("workspacer-config")
    }

    /// Ensures that the `.ws` directory exists, creating it if necessary.
    /// Returns an error if creation fails.
    pub fn ensure_dir_exists(&self) -> Result<(), WorkspacerFallbackError> {
        if !self.root.exists() {
            trace!(
                "WorkspacerDir not found at {:?}; creating it now.",
                self.root
            );
            create_dir_all(&self.root).map_err(|e| Arc::new(e))?;
            info!("Created WorkspacerDir at {:?}", self.root);
        }
        Ok(())
    }

    /// Creates (or ensures) a subdirectory within `.ws`. For instance:
    ///   `readme-writer-workspace`, `test-upgrader-workspace`, etc.
    pub fn ensure_subdir_exists(&self, subdir_name: &str) -> Result<PathBuf, WorkspacerFallbackError> {
        self.ensure_dir_exists()?;
        let subdir_path = self.root.join(subdir_name);
        if !subdir_path.exists() {
            trace!(
                "Subdirectory '{:?}' not found under {:?}; creating now.",
                subdir_path, self.root
            );
            create_dir_all(&subdir_path).map_err(|e| Arc::new(e))?;
            info!("Created subdir {:?} under {:?}", subdir_path, self.root);
        }
        Ok(subdir_path)
    }

    /// Removes the specified subdirectory entirely, including its contents,
    /// if it exists. This can be used for cleaning up caches, logs, etc.
    pub async fn remove_subdir(&self, subdir_name: &str) -> Result<(), WorkspacerFallbackError> {
        let subdir_path = self.root.join(subdir_name);
        if subdir_path.exists() {
            trace!("Removing subdirectory {:?} recursively.", subdir_path);
            tokio::fs::remove_dir_all(&subdir_path).await.map_err(|e| Arc::new(e))?;
            info!("Removed subdirectory {:?}", subdir_path);
        } else {
            debug!("Subdirectory {:?} does not exist; no action taken.", subdir_path);
        }
        Ok(())
    }

    /// Asynchronously loads the `workspacer-config` TOML from this `.ws` directory,
    /// returning `Ok(Some(WorkspacerConfig))` if found, `Ok(None)` if missing,
    /// or `Err(...)` on read/parse error.
    pub async fn load_config_async(&self) -> Result<Option<WorkspacerConfig>, WorkspacerFallbackError> {
        let file_path = self.config_file_path();
        trace!("Looking for config file at {:?}", file_path);

        match tokio::fs::read_to_string(&file_path).await {
            Ok(contents) => {
                trace!("Successfully read config file at {:?}", file_path);
                let parsed: WorkspacerConfig = toml::from_str(&contents)?;
                Ok(Some(parsed))
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    debug!("No config file found at {:?}", file_path);
                    Ok(None)
                } else {
                    error!("Error reading config file at {:?}: {:?}", file_path, e);
                    Err(Arc::new(e).into())
                }
            }
        }
    }

    /// A convenience method that tries to load the config from this `.ws` directory.
    /// If it doesn't exist (None), that's fine. If there's an I/O or parse error,
    /// returns an error. You can call this from your local or global dir easily.
    pub async fn load_or_create_config_async(
        &self,
    ) -> Result<WorkspacerConfig, WorkspacerFallbackError> {
        match self.load_config_async().await? {
            Some(cfg) => Ok(cfg),
            None => {
                // If no config file is present, we might decide to create a blank one
                // or return an error. For demonstration, we'll create an empty config.
                trace!("No config found; returning an empty config instance.");
                Ok(WorkspacerConfig::default())
            }
        }
    }
}
