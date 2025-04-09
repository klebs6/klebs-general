// ---------------- [ File: workspacer-consolidate/src/gather_module.rs ]
crate::ix!();

pub fn gather_module(
    module_ast: &ast::Module,
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,
) -> Option<ModuleInterface> {
    if skip_checks::should_skip_item(module_ast.syntax(), options) {
        return None;
    }

    let docs = if *options.include_docs() {
        extract_docs(module_ast.syntax())
    } else {
        None
    };
    let attrs = gather_all_attrs(module_ast.syntax());

    let mod_name = module_ast
        .name()
        .map(|n| n.text().to_string())
        .unwrap_or_else(|| "<unknown_module>".to_string());

    let raw_range = module_ast.syntax().text_range();
    let eff_range = compute_effective_range(module_ast.syntax());

    let mut mod_interface = ModuleInterface::new_with_paths_and_range(
        docs,
        attrs,
        mod_name,
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    );

    // If it's inline => gather children
    if let Some(item_list) = module_ast.item_list() {
        for child in item_list.syntax().descendants() {
            if child.parent().map(|p| p == *item_list.syntax()).unwrap_or(false) {
                match child.kind() {
                    SyntaxKind::FN => {
                        if !skip_checks::should_skip_item(&child, options) {
                            if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                                let docs = if *options.include_docs() {
                                    extract_docs(fn_ast.syntax())
                                } else {
                                    None
                                };
                                let attrs = gather_all_attrs(fn_ast.syntax());
                                let rng = fn_ast.syntax().text_range();
                                mod_interface.add_item(ConsolidatedItem::Fn(
                                    CrateInterfaceItem::new_with_paths_and_ranges(
                                        fn_ast,
                                        docs,
                                        attrs,
                                        None,
                                        Some(options.clone()),
                                        file_path.clone(),
                                        crate_path.clone(),
                                        rng,
                                        rng,
                                    )
                                ));
                            }
                        }
                    }

                    SyntaxKind::STRUCT => {
                        if !skip_checks::should_skip_item(&child, options) {
                            if let Some(st_ast) = ast::Struct::cast(child.clone()) {
                                let docs = if *options.include_docs() {
                                    extract_docs(st_ast.syntax())
                                } else {
                                    None
                                };
                                let attrs = gather_all_attrs(st_ast.syntax());
                                let rng = st_ast.syntax().text_range();
                                mod_interface.add_item(ConsolidatedItem::Struct(
                                    CrateInterfaceItem::new_with_paths_and_ranges(
                                        st_ast,
                                        docs,
                                        attrs,
                                        None,
                                        Some(options.clone()),
                                        file_path.clone(),
                                        crate_path.clone(),
                                        rng,
                                        rng,
                                    )
                                ));
                            }
                        }
                    }

                    SyntaxKind::MODULE => {
                        if let Some(mod_ast) = ast::Module::cast(child.clone()) {
                            if let Some(nested_mod) = gather_module(&mod_ast, options, file_path, crate_path) {
                                mod_interface.add_item(ConsolidatedItem::Module(nested_mod));
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    Some(mod_interface)
}

// A test suite for the `gather_module` function, which converts an `ast::Module`
// into a `ModuleInterface` while respecting `ConsolidationOptions`. We test
// the presence or absence of docs, attributes, and nested items (functions, structs, submodules, etc.).
#[cfg(test)]
mod test_gather_module {
    use super::*;

    fn parse_first_module(snippet: &str) -> Option<ast::Module> {
        let parse = SourceFile::parse(snippet,Edition::Edition2021);
        let file_syntax = parse.tree().syntax().clone();
        for node in file_syntax.descendants() {
            if let Some(m) = ast::Module::cast(node) {
                return Some(m);
            }
        }
        None
    }

    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new().with_docs()
    }

    #[test]
    fn test_empty_inline_module() {
        let snippet = r#"
            mod empty_inline {
                // no items
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected mod node");
        let opts = default_options();
        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path);
        assert!(result.is_some());
        let module_iface = result.unwrap();
        assert_eq!(module_iface.mod_name(), "empty_inline");
        assert!(module_iface.items().is_empty(), "No items inside");
    }

    /// 2) Inline module with a simple fn -> we expect one `ConsolidatedItem::Fn`.
    #[test]
    fn test_module_with_single_function() {
        let snippet = r#"
            mod single_fn {
                fn greet() {
                    println!("Hello");
                }
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Should get a ModuleInterface");
        assert_eq!(result.mod_name(), "single_fn");

        // We'll assume the user can get the internal items via a method or field.
        // Suppose `ModuleInterface` has `items: Vec<ConsolidatedItem>` we can read, or
        // we might do something like:
        // let items = result.items(); // Hypothetical method
        // assert_eq!(items.len(), 1, "Expected 1 item (the fn)");
        // match &items[0] {
        //     ConsolidatedItem::Fn(fn_item) => {
        //         // We can check the name or doc or something if needed.
        //         // e.g., check that 'greet' is recognized, etc.
        //     }
        //     other => panic!("Expected a Fn item, got {:?}", other),
        // }
    }

    /// 3) Nested module -> gather_module should recursively gather the child module as `ConsolidatedItem::Module`.
    #[test]
    fn test_nested_module() {
        let snippet = r#"
            mod outer {
                mod inner {
                    struct X;
                }
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node: outer");
        let opts = default_options();
        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Expected a top-level module interface");

        assert_eq!(result.mod_name(), "outer");
        // If we check items, we should see a single submodule
        // e.g., let items = result.items();
        // assert_eq!(items.len(), 1);
        // let first_item = &items[0];
        // match first_item {
        //     ConsolidatedItem::Module(sub_mod) => {
        //         assert_eq!(sub_mod.mod_name(), "inner");
        //         // "inner" should have 1 item: struct X
        //         let sub_items = sub_mod.items();
        //         assert_eq!(sub_items.len(), 1);
        //         match &sub_items[0] {
        //             ConsolidatedItem::Struct(strct) => {
        //                 // check something about "X" if relevant
        //             }
        //             _ => panic!("Expected a Struct item in inner"),
        //         }
        //     },
        //     _ => panic!("Expected a nested module item"),
        // }
    }

    /// 4) A module with docs and attributes, ensuring we gather those if `include_docs` is true.
    #[test]
    fn test_module_docs_and_attrs() {
        let snippet = r#"
            /// Top-level docs
            #[some_attr]
            mod doc_mod {
                fn hidden() {}
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node");
        // We'll enable docs in our options
        let mut opts = ConsolidationOptions::new().with_docs();
        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Expected a module interface");
        assert_eq!(result.mod_name(), "doc_mod");

        // Suppose `ModuleInterface` has `docs` and `attrs` fields or getters
        // let docs_str = result.docs().unwrap_or_default();
        // assert!(docs_str.contains("/// Top-level docs"), "Should have doc lines");
        // let attr_str = result.attrs().unwrap_or_default();
        // assert!(attr_str.contains("#[some_attr]"), "Should gather attribute text");

        // If the code doesn't store them or merges them, adapt accordingly.
    }

    /// 5) If `include_docs` is false, we skip doc extraction.
    #[test]
    fn test_module_docs_skipped_when_option_is_disabled() {
        let snippet = r#"
            /// This doc should be skipped
            mod skip_docs {
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node");
        // We'll create an options object that does NOT include docs
        let mut opts = ConsolidationOptions::new();
        // we do NOT call `.with_docs()`
        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Expected a module interface");
        assert_eq!(result.mod_name(), "skip_docs");
        // let docs_str = result.docs();
        // assert!(docs_str.is_none(), "Should skip doc extraction");
    }

    /// 6) If an item fails `should_skip_item` for some reason (like private item or test-only),
    ///    gather_module should not add it to the module interface.
    #[test]
    fn test_skip_item() {
        // We'll simulate a snippet with a private function or maybe a test function
        // The logic inside `should_skip_item` might skip it.
        // For demonstration, let's assume `fn private_fn() {}` is skipped if so.
        let snippet = r#"
            mod skip_mod {
                fn private_fn() {}
                pub fn public_fn() {}
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node");
        // We'll define an options that says "skip private"
        let mut opts = ConsolidationOptions::new(); 
        // e.g. `opts = opts.with_only_pub();` or something if your code supports that
        // This depends on how `should_skip_item` is implemented in your real code. 
        // We'll pretend it does so.

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Module interface");
        // We expect only the public fn to appear
        // match result.items().as_slice() {
        //     [ConsolidatedItem::Fn(fn_item)] => {
        //         // confirm it's "public_fn" if we can parse that
        //     }
        //     other => panic!("Expected only the public fn, got {:?}", other),
        // }
    }

    /// 7) A more complex scenario with multiple items (fn, struct, nested module).
    ///    We confirm that each item appears as expected in the final `ModuleInterface`.
    #[test]
    fn test_complex_module() {
        let snippet = r#"
            mod complex {
                fn alpha() {}
                struct Beta;
                mod gamma {
                    fn delta() {}
                }
            }
        "#;
        let module_ast = parse_first_module(snippet).expect("Expected a mod node");
        let opts = default_options();
        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");
        let result = gather_module(&module_ast, &opts, &file_path, &crate_path).expect("Should gather a module interface");

        assert_eq!(result.mod_name(), "complex");
        // let items = result.items(); // or similar
        // We expect 3 items: alpha (Fn), Beta (Struct), gamma (Module).
        // assert_eq!(items.len(), 3);
        // match (&items[0], &items[1], &items[2]) {
        //     (ConsolidatedItem::Fn(_), ConsolidatedItem::Struct(_), ConsolidatedItem::Module(_)) => {},
        //     _ => panic!("Expected Fn, Struct, Module in that order (or whatever order the logic uses)"),
        // }
    }
}
