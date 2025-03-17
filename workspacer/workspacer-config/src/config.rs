// ---------------- [ File: workspacer-config/src/config.rs ]
crate::ix!();

/// Represents the data we store in `.ws/workspacer-config` files, whether local or global.
///
/// You can add more fields as needed—like additional config relevant to
/// readme-writer-workspace, test-upgrader-workspace, etc. The key point is that
/// `WorkspacerConfig` itself *only* deals with the actual TOML data and does not
/// handle creating directories. That logic lives in `WorkspacerDir`.
#[derive(Debug, Clone, Builder, Getters, Setters, Serialize, Deserialize, Default)]
#[builder(setter(into), default)]
#[getset(get = "pub", set = "pub")]
pub struct WorkspacerConfig {
    authors:      Option<Vec<String>>,
    rust_edition: Option<String>,
    license:      Option<String>,
    repository:   Option<String>,
}

impl WorkspacerConfig {

    /// Tries to load the config from local `.ws`. If not found, tries global `.ws`.
    pub async fn load_with_fallback() -> Result<Option<WorkspacerConfig>, WorkspacerFallbackError> {
        let local_ws = WorkspacerDir::local();
        if let Some(cfg) = local_ws.load_config_async().await? {
            return Ok(Some(cfg));
        }
        // If local wasn't found, try global
        match WorkspacerDir::global() {
            Ok(global_ws) => {
                let global_cfg = global_ws.load_config_async().await?;
                Ok(global_cfg)
            }
            Err(e) => {
                warn!("Unable to find or create a global ws dir: {:?}", e);
                Ok(None)
            }
        }
    }
}

// --------------------------------------------
//     Example Tests for the .ws Directory
// --------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    use std::fs::{File, create_dir_all};

    #[tokio::test]
    async fn test_local_ws_dir_and_config() {
        // 1) Temp directory to simulate our project root
        let tmp = tempdir().expect("Failed to create temp dir");
        std::env::set_current_dir(tmp.path()).expect("Failed to set current dir");

        // 2) Construct local WorkspacerDir => "./.ws"
        let local_ws = WorkspacerDir::local();
        local_ws.ensure_dir_exists().expect("Could not create local .ws dir");

        // 3) Write a config file at `.ws/workspacer-config`
        let config_path = local_ws.config_file_path();
        let content = r#"
            authors = ["Test Author <test@example.com>"]
            rust_edition = "2021"
        "#;
        {
            let mut f = File::create(&config_path).expect("Failed to create config file");
            write!(f, "{}", content).expect("Failed to write config content");
        }

        // 4) Load config
        let cfg_opt = local_ws.load_config_async().await.expect("Failed to load config");
        assert!(cfg_opt.is_some(), "Expected some config after writing it.");
        let cfg = cfg_opt.unwrap();
        assert_eq!(
            cfg.authors().as_ref(),
            Some(&vec!["Test Author <test@example.com>".to_string()])
        );
        assert_eq!(cfg.rust_edition().as_ref(), Some(&"2021".to_string()));
    }

    #[traced_test]
    async fn test_subdirectory_creation() {
        // Just a sync example for subdir creation
        let tmp = tempdir().expect("Failed to create temp dir");
        std::env::set_current_dir(tmp.path()).expect("Failed to set current dir");

        let local_ws = WorkspacerDir::local();
        local_ws.ensure_dir_exists().unwrap();

        let subdir_path = local_ws.ensure_subdir_exists("readme-writer-workspace")
            .expect("Failed to create subdir");
        assert!(subdir_path.exists(), "Subdir should now exist");
        assert!(subdir_path.is_dir(), "Should be a directory");

        // Optionally remove it
        local_ws.remove_subdir("readme-writer-workspace").await.expect("Failed to remove subdir");
        assert!(!subdir_path.exists(), "Subdir should be removed now");
    }
}
