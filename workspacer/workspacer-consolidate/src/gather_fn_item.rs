// ---------------- [ File: src/gather_fn_item.rs ]
crate::ix!();

pub fn gather_fn_item(
    fn_ast:  &ast::Fn,
    options: &ConsolidationOptions,
) -> CrateInterfaceItem<ast::Fn> {

    let docs = if *options.include_docs() {
        extract_docs(fn_ast.syntax())
    } else {
        None
    };

    let attributes = gather_all_attrs(fn_ast.syntax());

    let is_test_item = is_in_test_module(fn_ast.syntax().clone()) || has_cfg_test_attr(fn_ast.syntax());

    let body_source = if is_test_item {
        if *options.include_fn_bodies_in_tests() {
            if let Some(block_expr) = fn_ast.body() {
                Some(block_expr.syntax().text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        if *options.include_fn_bodies() {
            if let Some(block_expr) = fn_ast.body() {
                Some(block_expr.syntax().text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    };

    CrateInterfaceItem::new(fn_ast.clone(), docs, attributes, body_source)
}

#[cfg(test)]
#[disable]
mod test_gather_fn_item {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxKind, SyntaxNode, Edition};

    // If your code references these from your crate, import them:
    // use crate::{
    //     gather_fn_item, // the function being tested
    //     ConsolidationOptions,
    //     CrateInterfaceItem, // the output type
    //     extract_docs, gather_all_attrs, is_in_test_module, has_cfg_test_attr,
    // };

    /// Helper: parse the Rust snippet into a `SyntaxNode`.
    fn parse_source(snippet: &str) -> SyntaxNode {
        // Some RA-AP versions want a second parameter Edition
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Finds the **first** `ast::Fn` in the syntax tree, if any.
    fn find_first_fn(root: &SyntaxNode) -> Option<ast::Fn> {
        for node in root.descendants() {
            if node.kind() == SyntaxKind::FN {
                if let Some(fn_ast) = ast::Fn::cast(node) {
                    return Some(fn_ast);
                }
            }
        }
        None
    }

    /// Creates a default `ConsolidationOptions` for these tests.
    /// Adjust to your real approach if you have specific toggles.
    fn default_options() -> ConsolidationOptions {
        // For example:
        ConsolidationOptions::new()
            .with_docs()
            .with_fn_bodies()
        // plus other flags as needed
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) A simple fn with no doc comments, no attributes, an empty body => we confirm the
    ///    returned item has no docs, no attributes, but the body_source is "{}".
    #[test]
    fn test_simple_fn_no_docs_no_attrs_with_body() {
        let snippet = r#"
            fn simple() {}
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);

        // The docs field should be None (no doc comments)
        assert_eq!(item.docs(), None, "No doc comments => no docs");
        // The attributes field should also be None (no attributes)
        assert_eq!(item.attributes(), None, "No attributes => None");
        // The body_source should be Some("{}") if `with_fn_bodies()` is set
        assert_eq!(item.body_source(), Some("{}".to_string()), "Empty body => {}");
    }

    /// 2) A function with doc comments and an attribute, confirm they appear in docs() and attributes().
    #[test]
    fn test_fn_with_docs_and_attributes() {
        let snippet = r#"
            /// This is a doc line
            #[inline]
            fn with_docs_and_inline() -> i32 { 42 }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let mut opts = default_options();
        // We keep docs, we keep fn bodies.

        let item = gather_fn_item(&fn_ast, &opts);

        // The doc text might look like "/// This is a doc line"
        // depending on how `extract_docs` was implemented, you might store or convert to a single string.
        let docs_opt = item.docs();
        assert!(docs_opt.is_some(), "We have doc lines");
        let docs = docs_opt.unwrap();
        assert!(
            docs.contains("/// This is a doc line"),
            "Docs should contain the line we wrote"
        );

        // The attributes might be "#[inline]".
        // `gather_all_attrs(fn_ast.syntax())` typically returns a string with the raw lines, or something similar.
        let attr_opt = item.attributes();
        assert!(attr_opt.is_some(), "We have an attribute");
        let attr = attr_opt.unwrap();
        assert!(
            attr.contains("#[inline]"),
            "Attributes should contain #[inline]"
        );

        // Check body
        let body_source = item.body_source();
        assert!(body_source.is_some(), "We have a body");
        let body_text = body_source.unwrap();
        assert_eq!(body_text, "{ 42 }", "Expected the function body text");
    }

    /// 3) A function that has no body (e.g., a trait method or extern "C" fn),
    ///    confirm that body_source is None even if `with_fn_bodies()` is true.
    #[test]
    fn test_fn_no_body() {
        let snippet = r#"
            extern "C" fn no_body_fn(param: i32);
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options(); // includes fn bodies

        let item = gather_fn_item(&fn_ast, &opts);

        // No doc or attributes
        assert_eq!(item.docs(), None);
        assert_eq!(item.attributes(), None);
        // body_source => None, because there's no block expression
        assert_eq!(item.body_source(), None, "No body => None body_source");
    }

    /// 4) If the function is in a test module or has `#[cfg(test)]`, we treat it differently:
    ///    if `include_fn_bodies_in_tests()` is off, we skip the body. If it's on, we include it.
    #[test]
    fn test_fn_in_test_module_or_cfg_test() {
        let snippet = r#"
            #[cfg(test)]
            fn test_fn() {
                println!("test!");
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        // We'll define two sets of options: one that includes fn bodies in tests, one that doesn't
        let mut opts_no_body_in_tests = ConsolidationOptions::new()
            .with_fn_bodies() // normal bodies
            .without_fn_bodies_in_tests(); // hypothetical method you may have

        let mut opts_including_test_bodies = ConsolidationOptions::new()
            .with_fn_bodies()
            .with_fn_bodies_in_tests();

        // 4a) When we skip test bodies
        let item_skip = gather_fn_item(&fn_ast, &opts_no_body_in_tests);
        assert_eq!(item_skip.body_source(), None, "Skipped test body");

        // 4b) When we include test bodies
        let item_include = gather_fn_item(&fn_ast, &opts_including_test_bodies);
        assert_eq!(
            item_include.body_source(),
            Some("{\n    println!(\"test!\");\n}".to_string())
        );
    }

    /// 5) A function in a test *module* (like `mod tests { fn something() {} }`) might also be recognized
    ///    by `is_in_test_module(fn_ast.syntax())`. We'll check that logic if your code handles that.
    #[test]
    fn test_fn_in_test_module() {
        let snippet = r#"
            #[cfg(test)]
            mod tests {
                fn some_test_fn() { println!("test in mod"); }
            }
        "#;
        // We'll parse, find the `fn some_test_fn()`, which is a child of the test module.
        // Then gather_fn_item checks if is_in_test_module => treat as test item if so.

        let root = parse_source(snippet);
        let fn_node = root.descendants().find_map(ast::Fn::cast).expect("Expected a fn");
        let mut opts = ConsolidationOptions::new().with_fn_bodies().without_fn_bodies_in_tests();

        // Because it's in a test module, `is_test_item = true`.
        // Hence we skip the body if `include_fn_bodies_in_tests == false`.
        let item = gather_fn_item(&fn_node, &opts);
        assert_eq!(item.body_source(), None, "Should skip body in test module by default");
    }

    /// 6) If docs are disabled in options, we skip doc extraction, etc.
    #[test]
    fn test_skip_docs_in_options() {
        let snippet = r#"
            /// doc comment
            fn skip_docs() {}
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let mut opts = ConsolidationOptions::new() // doesn't include docs
            .with_fn_bodies();

        let item = gather_fn_item(&fn_ast, &opts);
        // Confirm docs is None
        assert_eq!(item.docs(), None, "Docs are disabled in options => docs None");
        // body_source should be Some("{}")
        assert_eq!(item.body_source(), Some("{}".to_string()));
    }

    /// 7) If function has multiple attributes, they all appear in item.attributes().
    #[test]
    fn test_multiple_attributes() {
        let snippet = r#"
            #[inline]
            #[allow(dead_code)]
            fn multi_attr_fn() {
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = ConsolidationOptions::new().with_fn_bodies().with_docs(); // or whatever

        let item = gather_fn_item(&fn_ast, &opts);

        let attrs_opt = item.attributes();
        assert!(attrs_opt.is_some(), "Should have attributes");
        let attrs = attrs_opt.unwrap();
        assert!(
            attrs.contains("#[inline]"),
            "Should contain #[inline]"
        );
        assert!(
            attrs.contains("#[allow(dead_code)]"),
            "Should contain #[allow(dead_code)]"
        );
    }

    /// 8) If there's no block expression, e.g. `fn external_fn(param: i32);`, we confirm body_source is None.
    #[test]
    fn test_external_fn_no_block() {
        let snippet = r#"
            fn external_fn(param: i32);
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options(); 

        let item = gather_fn_item(&fn_ast, &opts);
        assert_eq!(item.body_source(), None, "No block => no body_source");
    }

    /// 9) A function with a multi-line block => body_source includes all those lines. 
    ///    This verifies that we capture the entire block expression text.
    #[test]
    fn test_multiline_body() {
        let snippet = r#"
            fn multiline_body() {
                let x = 10;
                println!("x = {}", x);
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = ConsolidationOptions::new().with_fn_bodies();

        let item = gather_fn_item(&fn_ast, &opts);
        let body_src = item.body_source().expect("We have a block");
        assert!(body_src.contains("let x = 10;"));
        assert!(body_src.contains("println!(\"x = {}\", x);"));
    }

    /// 10) We can also confirm that `gather_fn_item` doesn't break if the snippet includes a doc comment, attribute, etc.
    ///    This is effectively a combination of prior cases, but it’s good to have a fully loaded scenario.
    #[test]
    fn test_full_scenario_fn() {
        let snippet = r#"
            /// doc line
            #[my_attr]
            fn full_fn() {
                // multi-line body
                let msg = "Hello";
                println!("{}", msg);
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let mut opts = ConsolidationOptions::new()
            .with_docs()
            .with_fn_bodies()
            .with_fn_bodies_in_tests(); // in case it's test, not relevant here

        let item = gather_fn_item(&fn_ast, &opts);

        // docs
        let docs = item.docs().expect("Docs are included");
        assert!(docs.contains("/// doc line"), "Doc comment was captured");

        // attributes
        let attrs = item.attributes().expect("We have an attribute");
        assert!(attrs.contains("#[my_attr]"), "Should have #[my_attr]");

        // body
        let body_src = item.body_source().expect("We have a body");
        assert!(body_src.contains("let msg = \"Hello\";"));
        assert!(body_src.contains("println!("{}", msg);"));
    }
}
