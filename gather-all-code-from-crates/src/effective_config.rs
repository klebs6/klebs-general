crate::ix!();

#[derive(Debug, Clone, Builder, Getters, Setters)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct EffectiveConfig {

    #[builder(default)]
    crates: Vec<PathBuf>,

    #[builder(default)]
    criteria: AstFilterCriteria,
}

impl EffectiveConfig {

    pub fn from_cli_and_global(cli: &Cli, global_cfg: &GlobalConfig) -> AppResult<Self> {
        let mut criteria_builder = AstFilterCriteriaBuilder::default();
        criteria_builder
            .include_tests(*cli.include_tests() || *global_cfg.default_include_tests())
            .single_test_name(cli.single_test_name().clone())
            .omit_private(*cli.omit_private())
            .omit_bodies(*cli.omit_bodies() || *global_cfg.default_omit_bodies())
            .single_function_name(cli.single_function_name().clone())
            .excluded_files(cli.excluded_files().clone())
            .exclude_main_file(*cli.exclude_main_file())
            .remove_doc_comments(*cli.remove_doc_comments());

        let criteria = criteria_builder.build().map_err(|_|AppError::Config{reason:ErrorReason::Config})?;

        let crates = if !cli.crates().is_empty() {
            cli.crates().clone()
        } else {
            let cur = std::env::current_dir().map_err(|e|AppError::Io{code:e.kind()})?;
            vec![cur]
        };

        EffectiveConfigBuilder::default()
            .crates(crates)
            .criteria(criteria)
            .build()
            .map_err(|_|AppError::Config{reason:ErrorReason::Config})
    }
}

#[cfg(test)]
mod effective_config_tests {
    use super::*;

    #[test]
    fn test_merge_global_config_with_cli_single_test() {
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(false)
            .default_omit_bodies(false)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(true)
            .single_test_name(Some("test_me".to_string()))
            .omit_private(false)
            .omit_bodies(false)
            .single_function_name(None)
            .excluded_files(vec![])
            .exclude_main_file(false)
            .remove_doc_comments(false)
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        assert_eq!(*eff.criteria.include_tests(), true);
        assert_eq!(*eff.criteria.single_test_name(), Some("test_me".to_string()));
    }

    #[test]
    fn test_effective_config_from_cli_and_global() {
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(true)
            .default_omit_bodies(false)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(false)
            .single_test_name(None)
            .omit_private(true)
            .omit_bodies(false)
            .single_function_name(None)
            .excluded_files(vec![])
            .exclude_main_file(false)
            .remove_doc_comments(false)
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        // Global says include_tests=true, CLI says include_tests=false. By the code, we OR them:
        // The code: `.include_tests(*cli.include_tests() || *global_cfg.default_include_tests())`
        // CLI=false, global=true => final = true
        assert_eq!(*eff.criteria().include_tests(), true);
        assert_eq!(*eff.criteria().omit_private(), true); // from CLI
        assert_eq!(*eff.criteria().omit_bodies(), false); // global false, CLI false
    }

    #[test]
    fn test_effective_config_global_only() {
        // No CLI overrides, rely entirely on global defaults.
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(true)
            .default_omit_bodies(true)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(false) // Not used since global overrides (OR) logic
            .omit_bodies(false)   // Not used
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        // include_tests = true (global or CLI)
        // omit_bodies = true (global)
        assert_eq!(*eff.criteria().include_tests(), true);
        assert_eq!(*eff.criteria().omit_bodies(), true);
    }

    #[test]
    fn test_effective_config_cli_only_no_global() {
        // Global defaults are all false, CLI sets values directly.
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(false)
            .default_omit_bodies(false)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(true)
            .omit_private(true)
            .omit_bodies(true)
            .excluded_files(vec!["some_file.rs".to_string()])
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        assert_eq!(*eff.criteria().include_tests(), true);
        assert_eq!(*eff.criteria().omit_private(), true);
        assert_eq!(*eff.criteria().omit_bodies(), true);
        assert!(eff.criteria().excluded_files().contains(&"some_file.rs".to_string()));
    }

    #[test]
    fn test_effective_config_with_project_override() {
        // Suppose global config has a project-specific override that changes omit_private
        let mut project_overrides = HashMap::new();
        project_overrides.insert("mycrate".to_string(), {
            AstFilterCriteriaBuilder::default()
                .omit_private(true)
                .build().unwrap()
        });

        let global_cfg = GlobalConfigBuilder::default()
            .project_overrides(project_overrides)
            .default_include_tests(false)
            .default_omit_bodies(false)
            .build().unwrap();

        // CLI does not request omit_private, but global override does
        let cli = CliBuilder::default()
            .crates(vec![])
            .include_tests(false)
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        // There's currently no direct project key usage in from_cli_and_global, 
        // but if you ever integrate project-specific logic, this test is ready.
        // For now, this confirms that loading doesn't fail and that defaults still apply.
        assert_eq!(*eff.criteria().omit_private(), false, "No direct logic ties project overrides to CLI test, consider integrating logic if needed.");
    }

    #[test]
    fn test_effective_config_conflict_resolution() {
        // CLI says omit_bodies=false, global says default_omit_bodies=true
        let global_cfg = GlobalConfigBuilder::default()
            .default_omit_bodies(true)
            .default_include_tests(false)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .omit_bodies(false)
            .build()
            .unwrap();

        // The code sets omit_bodies to CLI value or global default. There's no OR logic here,
        // it's a direct assignment: `.omit_bodies(*cli.omit_bodies() || *global_cfg.default_omit_bodies())`
        // With cli.omit_bodies=false, global_omit_bodies=true => final = true (OR logic).
        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        assert_eq!(*eff.criteria().omit_bodies(), true); // Because true || false = true
    }

    #[test]
    fn test_effective_config_excluded_files_merge() {
        let global_cfg = GlobalConfigBuilder::default()
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .excluded_files(vec!["src/exclude_this.rs".to_string()])
            .build().unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        assert!(eff.criteria.excluded_files().contains(&"src/exclude_this.rs".to_string()));
    }

    #[test]
    fn test_effective_config_with_single_function_name() {
        let global_cfg = GlobalConfigBuilder::default()
            .default_include_tests(false)
            .build().unwrap();

        let cli = CliBuilder::default()
            .crates(vec![])
            .single_function_name(Some("specific_func".into()))
            .build()
            .unwrap();

        let eff = EffectiveConfig::from_cli_and_global(&cli, &global_cfg).unwrap();
        assert_eq!(*eff.criteria().single_function_name(), Some("specific_func".to_string()));
    }
}
