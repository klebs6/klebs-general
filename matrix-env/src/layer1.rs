// src/layer1.rs

crate::ix!();

#[derive(Debug)]
pub enum Layer1Error {
    EnvVarNotUnicode {
        key: &'static str,
    },
    MissingHomeDirectory,
    CurrentDirUnavailable {
        source: io::Error,
    },
    CreateDirAllFailed {
        path: PathBuf,
        source: io::Error,
    },
    ReadToStringFailed {
        path: PathBuf,
        source: io::Error,
    },
    TomlConfigDeserializeFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
    TomlSessionDeserializeFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
    TomlSessionSerializeFailed {
        source: toml::ser::Error,
    },
    OpenForAtomicWriteFailed {
        path: PathBuf,
        source: io::Error,
    },
    WriteAllFailed {
        path: PathBuf,
        source: io::Error,
    },
    SyncAllFailed {
        path: PathBuf,
        source: io::Error,
    },
    AtomicRenameFailed {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    RemoveFileFailed {
        path: PathBuf,
        source: io::Error,
    },
    SecretStorePathHasNoParent {
        path: PathBuf,
    },
    SetFilePermissionsFailed {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for Layer1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvVarNotUnicode { key } => write!(f, "environment variable is not valid unicode: {key}"),
            Self::MissingHomeDirectory => write!(f, "unable to resolve home directory (HOME not set, no explicit override)"),
            Self::CurrentDirUnavailable { source } => write!(f, "unable to resolve current directory: {source}"),
            Self::CreateDirAllFailed { path, source } => write!(f, "failed to create directory {:?}: {source}", path),
            Self::ReadToStringFailed { path, source } => write!(f, "failed to read file {:?}: {source}", path),
            Self::TomlConfigDeserializeFailed { path, source } => write!(f, "failed to parse TOML config {:?}: {source}", path),
            Self::TomlSessionDeserializeFailed { path, source } => write!(f, "failed to parse TOML session {:?}: {source}", path),
            Self::TomlSessionSerializeFailed { source } => write!(f, "failed to serialize session as TOML: {source}"),
            Self::OpenForAtomicWriteFailed { path, source } => write!(f, "failed to open temp file {:?} for atomic write: {source}", path),
            Self::WriteAllFailed { path, source } => write!(f, "failed to write all bytes to {:?}: {source}", path),
            Self::SyncAllFailed { path, source } => write!(f, "failed to sync file {:?}: {source}", path),
            Self::AtomicRenameFailed { from, to, source } => write!(f, "failed to atomically rename {:?} -> {:?}: {source}", from, to),
            Self::RemoveFileFailed { path, source } => write!(f, "failed to remove file {:?}: {source}", path),
            Self::SecretStorePathHasNoParent { path } => write!(f, "secret store path has no parent directory: {:?}", path),
            Self::SetFilePermissionsFailed { path, source } => write!(f, "failed to set permissions on {:?}: {source}", path),
        }
    }
}

impl std::error::Error for Layer1Error {}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Layer1AppPaths {
    app_name: String,
    config_dir: PathBuf,
    state_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Layer1AppPaths {
    fn resolve(spec: &Layer1LoadSpec) -> Result<Self, Layer1Error> {
        let app_name = spec.app_name().clone();

        let home = match spec.home_dir() {
            Some(h) => h.clone(),
            None => match std::env::var_os("HOME") {
                Some(v) => PathBuf::from(v),
                None => {
                    let current = std::env::current_dir()
                        .map_err(|e| Layer1Error::CurrentDirUnavailable { source: e })?;
                    warn!(
                        current_dir = %current.display(),
                        "HOME not set; using current directory as home fallback"
                    );
                    current
                }
            },
        };

        let config_base = spec
            .xdg_config_home()
            .clone()
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .unwrap_or_else(|| home.join(".config"));

        let state_base = spec
            .xdg_state_home()
            .clone()
            .or_else(|| std::env::var_os("XDG_STATE_HOME").map(PathBuf::from))
            .unwrap_or_else(|| home.join(".local").join("state"));

        let cache_base = spec
            .xdg_cache_home()
            .clone()
            .or_else(|| std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from))
            .unwrap_or_else(|| home.join(".cache"));

        Ok(Self {
            app_name: app_name.clone(),
            config_dir: config_base.join(&app_name),
            state_dir: state_base.join(&app_name),
            cache_dir: cache_base.join(&app_name),
        })
    }

    pub fn default_config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn default_session_file_path(&self) -> PathBuf {
        self.state_dir.join("session.toml")
    }

    pub fn ensure_state_dir_exists(&self) -> Result<(), Layer1Error> {
        ensure_dir_all_safely(&self.state_dir)
    }
}

#[derive(Default,Clone, Debug, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct Layer1LoadSpec {
    #[builder(default = "\"matrix-term\".to_string()")]
    app_name: String,
    home_dir: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    xdg_state_home: Option<PathBuf>,
    xdg_cache_home: Option<PathBuf>,
    config_path: Option<PathBuf>,
    session_path: Option<PathBuf>,
    config_toml: Option<String>,
    env_homeserver_url: Option<String>,
    env_user_id: Option<String>,
    env_device_id: Option<String>,
}

impl Layer1LoadSpec {
    pub fn for_process(app_name: &'static str) -> Result<Self, Layer1Error> {
        let config_path = std::env::var_os("MATRIX_TERM_CONFIG_PATH").map(PathBuf::from);
        let session_path = std::env::var_os("MATRIX_TERM_SESSION_PATH").map(PathBuf::from);

        let env_homeserver_url = first_env_string(&[
            "MATRIX_TERM_HOMESERVER_URL",
            "MATRIX_TERM_HOMESERVER",
        ])?;
        let env_user_id = first_env_string(&["MATRIX_TERM_USER_ID"])?;
        let env_device_id = first_env_string(&["MATRIX_TERM_DEVICE_ID"])?;

        Ok(Self {
            app_name: app_name.to_string(),
            home_dir: std::env::var_os("HOME").map(PathBuf::from),
            xdg_config_home: std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            xdg_state_home: std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
            xdg_cache_home: std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from),
            config_path,
            session_path,
            config_toml: None,
            env_homeserver_url,
            env_user_id,
            env_device_id,
        })
    }
}

#[derive(Default,Clone, Debug, Getters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into), default)]
pub struct Layer1RuntimeConfig {
    homeserver_url: Option<String>,
    user_id: Option<String>,
    device_id: Option<String>,
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Layer1LoadedConfig {
    paths: Layer1AppPaths,
    config_file_path: PathBuf,
    config: Layer1RuntimeConfig,
    secret_store: Layer1SecretStoreHandle,
}

pub struct Layer1ConfigLoader;

impl Layer1ConfigLoader {
    pub fn load_for_process() -> Result<Layer1LoadedConfig, Layer1Error> {
        let spec = Layer1LoadSpec::for_process("matrix-term")?;
        Self::load(spec)
    }

    pub fn load(spec: Layer1LoadSpec) -> Result<Layer1LoadedConfig, Layer1Error> {
        let paths = Layer1AppPaths::resolve(&spec)?;

        paths.ensure_state_dir_exists()?;

        let config_file_path = spec
            .config_path()
            .clone()
            .unwrap_or_else(|| paths.default_config_file_path());

        let file_cfg = match spec.config_toml() {
            Some(raw) => {
                debug!(
                    config_path = %config_file_path.display(),
                    "using config TOML override (in-memory)"
                );
                parse_file_config_from_str(&config_file_path, raw)?
            }
            None => read_optional_file_config(&config_file_path)?,
        };

        let session_path = spec
            .session_path()
            .clone()
            .or_else(|| file_cfg.session_path.clone())
            .unwrap_or_else(|| paths.default_session_file_path());

        let homeserver_url = spec
            .env_homeserver_url()
            .clone()
            .or(file_cfg.homeserver_url.clone());

        let user_id = spec.env_user_id().clone().or(file_cfg.user_id.clone());

        let device_id = spec
            .env_device_id()
            .clone()
            .or(file_cfg.device_id.clone());

        let cfg = Layer1RuntimeConfig {
            homeserver_url,
            user_id,
            device_id,
        };

        let secret_store = Layer1SecretStoreHandle::new_file(session_path);

        info!(
            app_name = %paths.app_name(),
            config_path = %config_file_path.display(),
            state_dir = %paths.state_dir().display(),
            cache_dir = %paths.cache_dir().display(),
            "layer1 resolved app paths"
        );

        if let Some(hs) = cfg.homeserver_url() {
            info!(homeserver_url = %hs, "layer1 homeserver configured");
        } else {
            warn!("layer1 homeserver not configured (expected for layer0/layer1 bring-up)");
        }

        if let Some(uid) = cfg.user_id() {
            info!(user_id = %uid, "layer1 user configured");
        } else {
            debug!("layer1 user not configured (expected for layer0/layer1 bring-up)");
        }

        Ok(Layer1LoadedConfig {
            paths,
            config_file_path,
            config: cfg,
            secret_store,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Layer1FileConfig {
    homeserver_url: Option<String>,
    user_id: Option<String>,
    device_id: Option<String>,
    session_path: Option<PathBuf>,
}

fn read_optional_file_config(path: &Path) -> Result<Layer1FileConfig, Layer1Error> {
    match fs::metadata(path) {
        Ok(_md) => {
            debug!(config_path = %path.display(), "reading config file");
            let raw = fs::read_to_string(path)
                .map_err(|e| Layer1Error::ReadToStringFailed { path: path.to_path_buf(), source: e })?;
            parse_file_config_from_str(path, &raw)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            debug!(config_path = %path.display(), "config file not found; continuing with defaults");
            Ok(Layer1FileConfig::default())
        }
        Err(e) => Err(Layer1Error::ReadToStringFailed {
            path: path.to_path_buf(),
            source: e,
        }),
    }
}

fn parse_file_config_from_str(path: &Path, raw: &str) -> Result<Layer1FileConfig, Layer1Error> {
    toml::from_str::<Layer1FileConfig>(raw).map_err(|e| Layer1Error::TomlConfigDeserializeFailed {
        path: path.to_path_buf(),
        source: e,
    })
}

fn ensure_dir_all_safely(path: &Path) -> Result<(), Layer1Error> {
    fs::create_dir_all(path).map_err(|e| Layer1Error::CreateDirAllFailed {
        path: path.to_path_buf(),
        source: e,
    })
}

fn first_env_string(keys: &[&'static str]) -> Result<Option<String>, Layer1Error> {
    for &k in keys {
        match std::env::var_os(k) {
            None => continue,
            Some(v) => {
                let s = v
                    .into_string()
                    .map_err(|_v| Layer1Error::EnvVarNotUnicode { key: k })?;
                return Ok(Some(s));
            }
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Layer1SecretStoreHandle {
    path: PathBuf,
}

impl Layer1SecretStoreHandle {
    pub fn new_file(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn read_session(&self) -> Result<Option<Layer1SessionSecrets>, Layer1Error> {
        match fs::metadata(&self.path) {
            Ok(_md) => {
                debug!(session_path = %self.path.display(), "reading session");
                let raw = fs::read_to_string(&self.path).map_err(|e| Layer1Error::ReadToStringFailed {
                    path: self.path.clone(),
                    source: e,
                })?;
                let parsed = toml::from_str::<Layer1SessionSecrets>(&raw).map_err(|e| {
                    Layer1Error::TomlSessionDeserializeFailed {
                        path: self.path.clone(),
                        source: e,
                    }
                })?;
                Ok(Some(parsed))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Layer1Error::ReadToStringFailed {
                path: self.path.clone(),
                source: e,
            }),
        }
    }

    pub fn write_session(&self, secrets: &Layer1SessionSecrets) -> Result<(), Layer1Error> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| Layer1Error::SecretStorePathHasNoParent {
                path: self.path.clone(),
            })?;

        ensure_dir_all_safely(parent)?;

        let encoded =
            toml::to_string(secrets).map_err(|e| Layer1Error::TomlSessionSerializeFailed { source: e })?;

        atomic_write_0600(&self.path, encoded.as_bytes())
    }

    pub fn clear_session(&self) -> Result<(), Layer1Error> {
        match fs::remove_file(&self.path) {
            Ok(()) => {
                info!(session_path = %self.path.display(), "session cleared");
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Layer1Error::RemoveFileFailed {
                path: self.path.clone(),
                source: e,
            }),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Layer1SessionSecrets {
    access_token: String,
    user_id: String,
    device_id: Option<String>,
}

impl Layer1SessionSecrets {
    pub fn new(access_token: String, user_id: String, device_id: Option<String>) -> Self {
        Self {
            access_token,
            user_id,
            device_id,
        }
    }
}

impl fmt::Debug for Layer1SessionSecrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layer1SessionSecrets")
            .field("access_token", &"<redacted>")
            .field("user_id", &self.user_id)
            .field("device_id", &self.device_id)
            .finish()
    }
}

fn atomic_write_0600(path: &Path, bytes: &[u8]) -> Result<(), Layer1Error> {
    let unique = unique_suffix();
    let tmp = path.with_extension(format!("tmp.{unique}"));

    trace!(
        final_path = %path.display(),
        tmp_path = %tmp.display(),
        bytes_len = bytes.len(),
        "atomic write starting"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| Layer1Error::OpenForAtomicWriteFailed {
                path: tmp.clone(),
                source: e,
            })?;

        file.write_all(bytes).map_err(|e| Layer1Error::WriteAllFailed {
            path: tmp.clone(),
            source: e,
        })?;

        file.sync_all().map_err(|e| Layer1Error::SyncAllFailed {
            path: tmp.clone(),
            source: e,
        })?;

        drop(file);

        fs::rename(&tmp, path).map_err(|e| Layer1Error::AtomicRenameFailed {
            from: tmp.clone(),
            to: path.to_path_buf(),
            source: e,
        })?;

        set_permissions_0600_if_possible(path)?;

        Ok(())
    }

    #[cfg(not(unix))]
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| Layer1Error::OpenForAtomicWriteFailed {
                path: tmp.clone(),
                source: e,
            })?;

        file.write_all(bytes).map_err(|e| Layer1Error::WriteAllFailed {
            path: tmp.clone(),
            source: e,
        })?;

        file.sync_all().map_err(|e| Layer1Error::SyncAllFailed {
            path: tmp.clone(),
            source: e,
        })?;

        drop(file);

        fs::rename(&tmp, path).map_err(|e| Layer1Error::AtomicRenameFailed {
            from: tmp.clone(),
            to: path.to_path_buf(),
            source: e,
        })?;

        Ok(())
    }
}

fn unique_suffix() -> String {
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_e| Duration::from_secs(0));

    format!("{pid}.{}.{}", now.as_secs(), now.subsec_nanos())
}

fn set_permissions_0600_if_possible(path: &Path) -> Result<(), Layer1Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let perm = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perm).map_err(|e| Layer1Error::SetFilePermissionsFailed {
            path: path.to_path_buf(),
            source: e,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod layer1_configuration_boundary_suite {
    use super::*;

    struct Layer1TempDir {
        path: PathBuf,
    }

    impl Layer1TempDir {
        fn new(stem: &'static str) -> Self {
            let base = std::env::temp_dir();
            let unique = unique_suffix();
            let path = base.join(format!("{stem}-{unique}"));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn join(&self, seg: &'static str) -> PathBuf {
            self.path.join(seg)
        }
    }

    impl Drop for Layer1TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[traced_test]
    fn layer1_load_uses_env_overrides_over_file_config() {
        let tmp = Layer1TempDir::new("layer1-env-overrides");

        let xdg_config_home = tmp.join("xdg-config");
        let xdg_state_home = tmp.join("xdg-state");
        let xdg_cache_home = tmp.join("xdg-cache");

        let raw = r#"
homeserver_url = "https://example.org"
user_id = "@file:example.org"
device_id = "FILEDEVICE"
"#;

        let spec = Layer1LoadSpecBuilder::default()
            .app_name("matrix-term")
            .home_dir(tmp.path().to_path_buf())
            .xdg_config_home(xdg_config_home)
            .xdg_state_home(xdg_state_home)
            .xdg_cache_home(xdg_cache_home)
            .config_toml(raw.to_string())
            .env_user_id("@env:example.org".to_string())
            .build()
            .expect("spec build");

        let loaded = Layer1ConfigLoader::load(spec).expect("layer1 load");

        assert_eq!(loaded.config().homeserver_url().as_deref(), Some("https://example.org"));
        assert_eq!(loaded.config().user_id().as_deref(), Some("@env:example.org"));
        assert_eq!(loaded.config().device_id().as_deref(), Some("FILEDEVICE"));
    }

    #[traced_test]
    fn layer1_load_succeeds_without_config_file() {
        let tmp = Layer1TempDir::new("layer1-no-config");

        let xdg_config_home = tmp.join("xdg-config");
        let xdg_state_home = tmp.join("xdg-state");
        let xdg_cache_home = tmp.join("xdg-cache");

        let spec = Layer1LoadSpecBuilder::default()
            .app_name("matrix-term")
            .home_dir(tmp.path().to_path_buf())
            .xdg_config_home(xdg_config_home)
            .xdg_state_home(xdg_state_home.clone())
            .xdg_cache_home(xdg_cache_home)
            .build()
            .expect("spec build");

        let loaded = Layer1ConfigLoader::load(spec).expect("layer1 load");

        assert_eq!(loaded.config().homeserver_url(), &None);
        assert_eq!(loaded.config().user_id(), &None);
        assert_eq!(loaded.config().device_id(), &None);

        assert!(
            loaded.paths().state_dir().starts_with(&xdg_state_home),
            "state dir should be under xdg_state_home"
        );

        assert!(
            fs::metadata(loaded.paths().state_dir()).is_ok(),
            "state dir should exist"
        );
    }

    #[traced_test]
    fn layer1_secret_store_file_roundtrip_and_clear() {
        let tmp = Layer1TempDir::new("layer1-secret-roundtrip");

        let session_path = tmp.join("state").join("session.toml");
        let store = Layer1SecretStoreHandle::new_file(session_path.clone());

        let secrets = Layer1SessionSecrets::new(
            "ACCESS_TOKEN_ABC123".to_string(),
            "@u:example.org".to_string(),
            Some("DEVICE1".to_string()),
        );

        store.write_session(&secrets).expect("write session");

        let got = store
            .read_session()
            .expect("read session")
            .expect("session should exist");

        assert_eq!(got, secrets);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let md = fs::metadata(&session_path).expect("metadata");
            let mode = md.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "session file should be 0600");
        }

        store.clear_session().expect("clear session");
        assert_eq!(store.read_session().expect("read after clear"), None);
    }
}
