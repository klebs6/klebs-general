crate::ix!();

#[derive(Debug, Clone, Deserialize, Builder, Getters, Setters)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct GlobalConfig {
    #[serde(default)]
    #[builder(default)]
    project_overrides: HashMap<String, AstFilterCriteria>,

    #[serde(default)]
    #[builder(default)]
    default_include_tests: bool,

    #[serde(default)]
    #[builder(default)]
    default_omit_bodies: bool,

    #[serde(default)]
    #[builder(default)]
    extra_flags: u64,
}

impl GlobalConfig {
    fn load_from_file(path: &PathBuf) -> AppResult<GlobalConfig> {
        if !path.exists() {
            return Ok(GlobalConfigBuilder::default().build().map_err(|_|AppError::Config{reason:ErrorReason::Config})?);
        }
        let file = File::open(path).map_err(|e|AppError::Io{code:e.kind()})?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).map_err(|e|AppError::Io{code:e.kind()})?;
        let cfg: GlobalConfig = serde_json::from_str(&content).map_err(|_|
            AppError::Config{reason:ErrorReason::Parse}
        )?;
        Ok(cfg)
    }
}

pub fn load_global_config() -> AppResult<GlobalConfig> {
    let home = std::env::var("HOME").map_err(|_|AppError::Config{reason:ErrorReason::MissingData})?;
    let config_path = PathBuf::from(home).join(".gather-all-code-from-crates");
    GlobalConfig::load_from_file(&config_path)
}

#[cfg(test)]
mod global_config_tests {
    use super::*;

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
        // include_tests should be true from global default OR cli (cli=false, global=true => result = true)
        assert_eq!(*eff.criteria().include_tests(), true);
        assert_eq!(*eff.criteria().omit_private(), true); // from cli
        assert_eq!(*eff.criteria().omit_bodies(), false); // global false, cli false
    }

    #[test]
    fn test_global_config_load_defaults_if_missing() {
        let path = PathBuf::from("non_existent_config_file.json");
        let cfg = GlobalConfig::load_from_file(&path).unwrap();
        assert_eq!(cfg.project_overrides().len(), 0);
        assert_eq!(*cfg.default_include_tests(), false);
        assert_eq!(*cfg.default_omit_bodies(), false);
    }

    #[test]
    fn test_global_config_load_from_file() {
        let tmp_dir = TempDir::new().unwrap();
        let cfg_path = tmp_dir.path().join("config.json");
        let content = r#"
        {
          "project_overrides": {
            "mycrate": {
              "include_tests": true,
              "omit_private": true
            }
          },
          "default_include_tests": true,
          "default_omit_bodies": false,
          "extra_flags": 42
        }
        "#;

        {
            let mut f = File::create(&cfg_path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }

        let cfg = GlobalConfig::load_from_file(&cfg_path).unwrap();
        assert_eq!(cfg.project_overrides().len(), 1);
        assert!(cfg.project_overrides().get("mycrate").unwrap().include_tests());
        assert!(cfg.project_overrides().get("mycrate").unwrap().omit_private());
        assert_eq!(*cfg.default_include_tests(), true);
        assert_eq!(*cfg.default_omit_bodies(), false);
        assert_eq!(*cfg.extra_flags(), 42);
    }

    #[test]
    fn test_global_config_load_failure_on_invalid_json() {
        let tmp_dir = TempDir::new().unwrap();
        let cfg_path = tmp_dir.path().join("config.json");
        {
            let mut f = File::create(&cfg_path).unwrap();
            f.write_all(b"{ invalid json }").unwrap();
        }

        let result = GlobalConfig::load_from_file(&cfg_path);
        match result {
            Err(AppError::Config { reason: ErrorReason::Parse }) => (),
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_global_config_missing_home_env() {
        // Temporarily unset HOME and ensure error is returned
        let original_home = std::env::var_os("HOME");
        std::env::remove_var("HOME");

        let result = load_global_config();
        match result {
            Err(AppError::Config{reason:ErrorReason::MissingData}) => (),
            _ => panic!("Expected MissingData error when HOME is unset"),
        }

        // Restore HOME
        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        }
    }
}
