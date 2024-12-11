crate::ix!();

pub fn process_file(path: &PathBuf, criteria: &AstFilterCriteria) -> Result<String, AppError> {
    let content = fs::read_to_string(path).map_err(|e|AppError::Io{code:e.kind()})?;
    let parsed = ra_ap_syntax::SourceFile::parse(&content, ra_ap_syntax::Edition::CURRENT);
    let parse_errors = parsed.errors();
    if !parse_errors.is_empty() {
        for err in parse_errors {
            eprintln!("Parsing error in {}: {:?}", path.display(), err);
        }
        return Err(AppError::Parse { reason: ErrorReason::Parse });
    }

    let syntax = parsed.tree().syntax().clone();
    let items  = extract_items_from_ast(&syntax, *criteria.remove_doc_comments());

    // Apply filtering
    let mut filtered_items = items;

    // Filter by include_tests
    if !criteria.include_tests() {
        filtered_items = filtered_items.into_iter().filter(|item| {
            match item {
                ItemInfo::Function(f) => !f.is_test(),
                ItemInfo::ImplBlock { methods, .. } => {
                    // Remove test methods
                    let filtered: Vec<FunctionInfo> = methods.iter().filter(|m| !m.is_test()).cloned().collect();
                    filtered.len() > 0 // Keep impl block only if there's something left (or return all methods filtered)
                }
                _ => true // Non-function items are not tests
            }
        }).map(|mut item| {
            if let ItemInfo::ImplBlock { methods, .. } = &mut item {
                *methods = methods.iter().filter(|m| !m.is_test()).cloned().collect();
            }
            item
        }).collect();
    }

    // Filter by single_test_name
    if let Some(test_name) = criteria.single_test_name() {
        filtered_items = filtered_items.into_iter().filter(|item| {
            match item {
                ItemInfo::Function(f) => *f.is_test() && f.name() == test_name,
                ItemInfo::ImplBlock { methods, .. } => {
                    // Keep only methods matching test_name and is_test
                    methods.iter().any(|m| *m.is_test() && m.name() == test_name)
                }
                _ => true
            }
        }).map(|mut item| {
            if let ItemInfo::ImplBlock { methods, .. } = &mut item {
                *methods = methods.iter().filter(|m| *m.is_test() && m.name() == test_name).cloned().collect();
            }
            item
        }).collect();
    }

    // Filter by omit_private
    if *criteria.omit_private() {
        filtered_items = filtered_items.into_iter().filter(|item| {
            match item {
                ItemInfo::Function(f) => *f.is_public(),
                ItemInfo::Struct { is_public, .. } => *is_public,
                ItemInfo::Enum { is_public, .. } => *is_public,
                ItemInfo::TypeAlias { is_public, .. } => *is_public,
                ItemInfo::ImplBlock { is_public, methods, ..} => {
                    // Impl blocks themselves have no pub, but we can decide to keep them if they have public methods
                    methods.iter().any(|m| *m.is_public())
                }
            }
        }).map(|mut item| {
            if let ItemInfo::ImplBlock { methods, .. } = &mut item {
                *methods = methods.iter().filter(|m| *m.is_public()).cloned().collect();
            }
            item
        }).collect();
    }

    // Filter by single_function_name
    if let Some(func_name) = criteria.single_function_name() {
        filtered_items = filtered_items.into_iter().filter(|item| {
            match item {
                ItemInfo::Function(f) => f.name() == func_name,
                ItemInfo::ImplBlock { methods, .. } => methods.iter().any(|m| m.name() == func_name),
                _ => true
            }
        }).map(|mut item| {
            if let ItemInfo::ImplBlock { methods, .. } = &mut item {
                *methods = methods.iter().filter(|m| m.name() == func_name).cloned().collect();
            }
            item
        }).collect();
    }

    let omit_bodies = *criteria.omit_bodies() && criteria.single_function_name().is_none();

    let reconstructed = reconstruct_code_from_filtered_items(&filtered_items, omit_bodies);

    Ok(reconstructed)
}

#[cfg(test)]
mod process_file_tests {
    use super::*;

    #[test]
    fn test_process_file_basic() {
        let code = r#"
            pub fn visible() {}
            fn hidden() {}
            #[test]
            fn test_something() {}
        "#;
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test.rs");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(code.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default()
            .include_tests(false)
            .omit_private(true)
            .build().unwrap();

        let result = process_file(&file_path, &criteria).unwrap();
        // include_tests=false: test_something filtered out
        // omit_private=true: hidden filtered out
        // only visible remains
        assert!(result.contains("pub fn visible()"));
        assert!(!result.contains("test_something"));
        assert!(!result.contains("hidden"));
    }

    #[test]
    fn test_process_file_single_function_name() {
        let code = r#"
            pub fn visible() {}
            pub fn visible2() {}
        "#;
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test2.rs");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(code.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default()
            .single_function_name(Some("visible".to_string()))
            .build().unwrap();

        let result = process_file(&file_path, &criteria).unwrap();
        assert!(result.contains("fn visible("));
        assert!(!result.contains("visible2"));
    }


    #[test]
    fn test_process_file_remove_doc_comments() {
        let code = r#"
            #[inline]
            #[test]
            fn mytest() {}
        "#;
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test3.rs");
            {
                let mut f = File::create(&file_path).unwrap();
                f.write_all(code.as_bytes()).unwrap();
            }

            // Enable tests so that `#[test]` functions are not filtered out.
            let criteria = AstFilterCriteriaBuilder::default()
                .remove_doc_comments(true)
                .include_tests(true) // <-- Add this line
                .build().unwrap();

        println!("criteria: {:#?}", criteria);

        let result = process_file(&file_path, &criteria).unwrap();
        println!("result: {:#?}", result);

        // Now that tests are included, the function `mytest` will remain, as will its attributes.
        assert!(result.contains("#[inline]"));
        assert!(result.contains("#[test]"));
    }

    #[test]
    fn test_process_file_syntax_error() {
        let code = r#"
            fn broken( {}
        "#;
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("broken.rs");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(code.as_bytes()).unwrap();
        }

        let criteria = AstFilterCriteriaBuilder::default().build().unwrap();
        let result = process_file(&file_path, &criteria);
        // This should fail due to parse errors
        match result {
            Err(AppError::Parse { reason: ErrorReason::Parse }) => (),
            _ => panic!("Expected parse error"),
        }
    }
}
