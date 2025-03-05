// ---------------- [ File: workspacer-consolidate/src/gather_fn_item.rs ]
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

    CrateInterfaceItem::new(fn_ast.clone(), docs, attributes, body_source, Some(options.clone()))
}

#[cfg(test)]
mod test_gather_fn_item {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxKind, SyntaxNode, Edition};

    /// A helper to parse a snippet into its root `SyntaxNode`.
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Finds the first `ast::Fn` in the syntax tree, if any.
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

    /// Our default test options: docs + normal bodies. 
    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new()
            .with_docs()
            .with_fn_bodies()
    }

    // ------------------------------------------------------------------------
    // The test suite
    // ------------------------------------------------------------------------

    #[test]
    fn test_simple_fn_no_docs_no_attrs_with_body() {
        let snippet = r#"
            fn simple() {
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        assert_eq!(*item.docs(), None, "No doc => none");
        assert_eq!(*item.attributes(), None, "No attrs => none");

        let raw_body = item.body_source().clone().expect("Should have body");
        let simplified = raw_body.replace(char::is_whitespace, "");
        assert_eq!(simplified, "{}", "Empty body => {{}}");
    }

    #[test]
    fn test_fn_with_docs_and_attributes() {
        let snippet = r#"
            /// This is a doc line
            #[inline]
            fn with_docs_and_inline() -> i32 { 42 }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        let docs = item.docs().clone().expect("Should have docs");
        assert!(docs.contains("/// This is a doc line"));

        let attr = item.attributes().clone().expect("Should have attrs");
        assert!(attr.contains("#[inline]"));

        let body_source = item.body_source().clone().expect("Should have body");
        assert_eq!(body_source, "{ 42 }");
    }

    #[test]
    fn test_fn_no_body() {
        let snippet = r#"
            extern "C" fn no_body_fn(param: i32);
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        assert_eq!(*item.docs(), None);
        assert_eq!(*item.attributes(), None);
        assert_eq!(*item.body_source(), None, "No block => no body");
    }

    #[test]
    fn test_fn_in_test_module_or_cfg_test() {
        let snippet = r#"
            #[cfg(test)]
            fn test_fn() {
                println!("test!");
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");

        let opts_no_body_in_tests = ConsolidationOptions::new().with_fn_bodies();
        let opts_with_test_bodies = ConsolidationOptions::new()
            .with_fn_bodies()
            .with_fn_bodies_in_tests();

        // (a) skip body if .with_fn_bodies_in_tests() is off
        let item_skip = gather_fn_item(&fn_ast, &opts_no_body_in_tests);
        assert_eq!(*item_skip.body_source(), None);

        // (b) gather body if .with_fn_bodies_in_tests() is on
        let item_include = gather_fn_item(&fn_ast, &opts_with_test_bodies);
        let actual_body = item_include.body_source().clone().expect("Should have body");
        let normalized = actual_body.replace(char::is_whitespace, "");
        assert_eq!(normalized, "{println!(\"test!\");}");
    }

    #[test]
    fn test_fn_in_test_module() {
        let snippet = r#"
            #[cfg(test)]
            mod tests {
                fn some_test_fn() { println!("test in mod"); }
            }
        "#;
        let root = parse_source(snippet);
        let fn_node = root.descendants().find_map(ast::Fn::cast).expect("Expected fn");
        let opts = default_options(); // no .with_fn_bodies_in_tests() => skip

        let item = gather_fn_item(&fn_node, &opts);
        assert_eq!(*item.body_source(), None, "Should skip body in test mod by default");
    }

    #[test]
    fn test_skip_docs_in_options() {
        let snippet = r#"
            /// doc comment
            fn skip_docs() {
                // we put something in the body so we see a real block
                let _z = 123;
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");

        // We do *not* call .with_docs() => docs are off. 
        // But we do want the function body => .with_fn_bodies().
        let opts = ConsolidationOptions::new()
            .with_fn_bodies()
            .with_fn_bodies_in_tests(); // so we definitely see the body

        let item = gather_fn_item(&fn_ast, &opts);
        // Confirm docs => None
        assert_eq!(*item.docs(), None, "Docs are disabled => none");

        // Confirm we do have a body
        let raw_body = item.body_source().clone().expect("Should have body");
        let simplified = raw_body.replace(char::is_whitespace, "");
        // Should contain the let statement if we want to be sure
        assert!(simplified.contains("let_z=123;"), "We see some real body");
    }

    #[test]
    fn test_multiple_attributes() {
        let snippet = r#"
            #[inline]
            #[allow(dead_code)]
            fn multi_attr_fn() {
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        let attrs = item.attributes().clone().expect("We have attrs");
        assert!(attrs.contains("#[inline]"));
        assert!(attrs.contains("#[allow(dead_code)]"));
    }

    #[test]
    fn test_external_fn_no_block() {
        let snippet = r#"
            fn external_fn(param: i32);
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        assert_eq!(*item.body_source(), None, "No block => no body");
    }

    #[test]
    fn test_multiline_body() {
        let snippet = r#"
            fn multiline_body() {
                let x = 10;
                println!("x = {}", x);
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected fn");
        let opts = default_options();

        let item = gather_fn_item(&fn_ast, &opts);
        let body_src = item.body_source().clone().expect("We have a block");
        assert!(body_src.contains("let x = 10;"));
        assert!(body_src.contains("println!(\"x = {}\", x);"));
    }

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
        let fn_ast = find_first_fn(&root).expect("Expected fn");
        let opts = default_options().with_fn_bodies_in_tests();

        let item = gather_fn_item(&fn_ast, &opts);

        // docs
        let docs = item.docs().clone().expect("Should have doc line");
        assert!(docs.contains("/// doc line"));

        // attributes
        let attrs = item.attributes().clone().expect("Should have attribute");
        assert!(attrs.contains("#[my_attr]"));

        // body
        let body_src = item.body_source().clone().expect("Should have body");
        assert!(body_src.contains("let msg = \"Hello\";"));
        assert!(body_src.contains("println!(\"{}\", msg);"));
    }
}
