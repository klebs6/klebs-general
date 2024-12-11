crate::ix!();

pub fn process_crate_directory(dir: &PathBuf, criteria: &AstFilterCriteria) -> AppResult<String> {
    if !dir.exists() {
        return Err(AppError::InvalidInput{reason:ErrorReason::MissingData});
    }

    let mut combined = String::new();
    let mut dirs_to_visit = vec![dir.clone()];
    while let Some(d) = dirs_to_visit.pop() {
        let entries = fs::read_dir(&d).map_err(|e|AppError::Io{code:e.kind()})?;
        for entry in entries {
            let entry = entry.map_err(|e|AppError::Io{code:e.kind()})?;
            let p = entry.path();

            if !p.starts_with(dir.join("src")) {
                continue; // only process files in src
            }

            if p.is_dir() {
                dirs_to_visit.push(p);
            } else {
                if p.extension().and_then(|s|s.to_str()) == Some("rs") {
                    let relative = match p.strip_prefix(dir) {
                        Ok(rel) => rel.display().to_string(),
                        Err(_) => p.display().to_string(),
                    };
                    if criteria.excluded_files().iter().any(|ex| ex == &relative) {
                        continue;
                    }

                    // If exclude_main_file is true and this is src/main.rs, skip
                    if *criteria.exclude_main_file() && relative == "src/main.rs" {
                        continue;
                    }

                    let snippet = process_file(&p, criteria)?;
                    if !snippet.trim().is_empty() {
                        combined.push_str(&snippet);
                        combined.push('\n');
                    }
                }
            }
        }
    }

    Ok(combined)
}

#[cfg(test)]
mod process_crate_directory_tests {
    use super::*;

    #[test]
    fn test_process_crate_directory_basic() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        let lib_path = src_dir.join("lib.rs");
        let code_lib = r#"pub fn lib_func() {} fn priv_func() {}"#;
        {
            let mut f = File::create(&lib_path).unwrap();
            f.write_all(code_lib.as_bytes()).unwrap();
        }

        let mod_path = src_dir.join("mod.rs");
        let code_mod = r#"#[test] fn test_in_mod() {} fn normal_in_mod() {}"#;
        {
            let mut f = File::create(&mod_path).unwrap();
            f.write_all(code_mod.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default()
            .include_tests(false)
            .omit_private(true)
            .build().unwrap();

        let result = process_crate_directory(&tmp_dir.path().to_path_buf(), &criteria).unwrap();
        // From lib.rs: omit_private=true leaves only pub fn lib_func
        // From mod.rs: tests excluded, so test_in_mod gone, normal_in_mod private => gone
        assert!(result.contains("pub fn lib_func()"));
        assert!(!result.contains("priv_func"));
        assert!(!result.contains("test_in_mod"));
        assert!(!result.contains("normal_in_mod"));
    }

    #[test]
    fn test_process_crate_directory_excluded_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        let keep_path = src_dir.join("keep.rs");
        let exclude_path = src_dir.join("exclude.rs");

        let code_keep = r#"pub fn keep_me() {}"#;
        {
            let mut f = File::create(&keep_path).unwrap();
            f.write_all(code_keep.as_bytes()).unwrap();
        }

        let code_exclude = r#"pub fn exclude_me() {}"#;
        {
            let mut f = File::create(&exclude_path).unwrap();
            f.write_all(code_exclude.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default()
            .excluded_files(vec!["src/exclude.rs".to_string()])
            .build().unwrap();

        let result = process_crate_directory(&tmp_dir.path().to_path_buf(), &criteria).unwrap();
        assert!(result.contains("keep_me"));
        assert!(!result.contains("exclude_me"));
    }

    #[test]
    fn test_process_crate_directory_exclude_main_file() {
        let tmp_dir = TempDir::new().unwrap();
        let src_dir = tmp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        let main_path = src_dir.join("main.rs");
        let lib_path = src_dir.join("lib.rs");

        let code_main = r#"fn main() {}"#;
        {
            let mut f = File::create(&main_path).unwrap();
            f.write_all(code_main.as_bytes()).unwrap();
        }

        let code_lib = r#"pub fn visible() {}"#;
        {
            let mut f = File::create(&lib_path).unwrap();
            f.write_all(code_lib.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default()
            .exclude_main_file(true)
            .build().unwrap();

        info!("criteria: {:#?}", criteria);

        let result = process_crate_directory(&tmp_dir.path().to_path_buf(), &criteria).unwrap();

        info!("result: {:#?}", result);

        // main.rs excluded
        assert!(result.contains("visible"));
        assert!(!result.contains("main()"));
    }

    #[test]
    fn test_error_handling_invalid_input() {
        let path = PathBuf::from("non_existent_dir");
        let criteria = AstFilterCriteriaBuilder::default().build().unwrap();
        let result = process_crate_directory(&path, &criteria);
        match result {
            Err(AppError::InvalidInput { reason: ErrorReason::MissingData }) => (),
            _ => panic!("Expected MissingData error"),
        }
    }
}
