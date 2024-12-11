crate::ix!();

/// Attempts to load global configuration and merge it with CLI arguments to produce an EffectiveConfig.
/// Returns `Result<EffectiveConfig, AppError>` if successful, or `AppError` if something goes wrong.
pub fn build_effective_config_from_cli() -> Result<EffectiveConfig, AppError> {
    let cli = Cli::from_args();

    // Try to load global config, fallback to defaults if needed
    let global_cfg = match load_global_config() {
        Ok(cfg) => cfg,
        Err(_e) => {
            // If config load fails, use defaults
            GlobalConfigBuilder::default()
                .build()
                .map_err(|_| AppError::Config { reason: ErrorReason::Config })?
        }
    };

    // Merge CLI and global config into EffectiveConfig
    EffectiveConfig::from_cli_and_global(&cli, &global_cfg)
}

#[cfg(test)]
mod build_effective_config_from_cli_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_effective_config_with_mocked_cli_and_global() {
        // Prepare a mock global config
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(true)
            .default_omit_bodies(false)
            .build().unwrap();

        // Prepare a mock CLI. Since Cli::from_args() is used at runtime,
        // For testing, we can directly build a Cli instance using the builder if available.
        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(false)
            .omit_private(true)
            .omit_bodies(false)
            .excluded_files(vec!["some_file.rs".to_string()])
            .build()
            .unwrap();

        // Merge them
        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();

        // Assert conditions
        // include_tests = true from global OR cli (global=true, cli=false => true)
        assert_eq!(*eff.criteria().include_tests(), true);
        // omit_private = true from cli
        assert_eq!(*eff.criteria().omit_private(), true);
        // omit_bodies = global=false, cli=false => false OR false => false
        assert_eq!(*eff.criteria().omit_bodies(), false);
        // Excluded files from cli are present
        assert!(eff.criteria().excluded_files().contains(&"some_file.rs".to_string()));
    }

    #[test]
    fn test_build_effective_config_from_cli_with_no_global_config() {
        // Temporarily remove HOME env var to force global config load to fail
        let original_home = std::env::var_os("HOME");
        std::env::remove_var("HOME");

        // Simulate CLI arguments using `from_iter`
        let args = vec!["myprog", "--include-tests", "--omit-private"];
        let cli = Cli::from_iter(args);

        // Simulate global config load failure and fallback to default
        let global_cfg = match GlobalConfigBuilder::default().build() {
            Ok(cfg) => cfg,
            Err(_) => panic!("Failed to build default global config"),
        };

        // Now call the logic for building the effective config
        let result = EffectiveConfig::from_cli_and_global(&cli, &global_cfg);
        assert!(result.is_ok());
        let eff = result.unwrap();

        // From CLI: include_tests=true, omit_private=true
        assert_eq!(*eff.criteria().include_tests(), true);
        assert_eq!(*eff.criteria().omit_private(), true);

        // Restore HOME if it was set
        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        }
    }
}
