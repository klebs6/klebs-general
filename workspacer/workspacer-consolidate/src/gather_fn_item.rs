// ---------------- [ File: workspacer-consolidate/src/gather_fn_item.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn gather_fn_item(
    fn_ast:  &ast::Fn,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> CrateInterfaceItem<ast::Fn> {
    trace!("Entering gather_fn_item; fn snippet = {:?}", snippet_for_logging(fn_ast.syntax()));

    // 1) Possibly gather doc lines
    let direct_docs = if *options.include_docs() {
        extract_docs(fn_ast.syntax())
    } else {
        None
    };

    // 2) Gather attributes
    let attributes = gather_all_attrs(fn_ast.syntax());

    // 3) Decide if we want the function body
    let is_test_item = is_in_test_module(fn_ast.syntax().clone()) || has_cfg_test_attr(fn_ast.syntax());
    let include_body = if is_test_item {
        *options.include_fn_bodies_in_tests()
    } else {
        *options.include_fn_bodies()
    };

    // 4) If we do want a body, either `fn_ast.body()` or a fallback.
    //
    //    Some RA parse scenarios (especially with multi-line `where` clauses)
    //    can produce a “body” node that is actually empty or missing the real statements.
    //    In that case, we also attempt `fallback_extract_fn_body(fn_ast)` if the parsed body
    //    looks suspiciously empty (`"{}"`).
    let mut body_source = None;
    if include_body {
        let parsed_body = fn_ast.body().map(|b| b.syntax().text().to_string());
        let final_body = match parsed_body {
            Some(ref bstr) if bstr.trim() == "{}" => {
                trace!("Parsed body is just an empty block => try fallback_extract_fn_body");
                fallback_extract_fn_body(fn_ast)
            }
            Some(non_empty) => {
                trace!("Parsed body is non-empty; using raw text from fn_ast.body() {:#?}", non_empty);
                Some(non_empty)
            }
            None => {
                trace!("fn_ast.body() is None => attempt fallback_extract_fn_body(fn_ast)");
                fallback_extract_fn_body(fn_ast)
            }
        };
        body_source = final_body;
    }

    // 5) Build final CrateInterfaceItem using raw + effective range
    let raw_range = fn_ast.syntax().text_range();
    let eff_range = compute_effective_range(fn_ast.syntax());

    trace!(
        "Constructing CrateInterfaceItem<ast::Fn> with raw_range={:?}, effective_range={:?}",
        raw_range,
        eff_range
    );

    CrateInterfaceItem::new_with_paths_and_ranges(
        fn_ast.clone(),
        direct_docs,
        attributes,
        body_source,
        Some(options.clone()),
        file_path.clone(),
        crate_path.clone(),
        raw_range,
        eff_range,
    )
}

/// Allow scanning **any** amount of whitespace (including multiple newlines & indentation)
/// as we go upward from `node.first_token()`, attaching consecutive `///` or `//!` lines.
/// Stops only if we encounter non‐doc comments or other tokens.
#[tracing::instrument(level = "trace", skip(node))]
fn gather_preceding_doc_comments(node: &SyntaxNode) -> Option<String> {
    let first_token = node.first_token()?;
    let mut lines = Vec::new();
    let mut tok_opt = first_token.prev_token();

    while let Some(tok) = tok_opt {
        match tok.kind() {
            SyntaxKind::WHITESPACE => {
                // Just skip whitespace and keep going. We don’t break on multiple newlines anymore.
            }
            SyntaxKind::COMMENT => {
                let text = tok.text().trim_start();
                if text.starts_with("///") || text.starts_with("//!") {
                    lines.push(text.to_string());
                } else {
                    // normal // comment => break
                    break;
                }
            }
            _ => {
                // any other token => stop
                break;
            }
        }
        tok_opt = tok.prev_token();
    }

    if lines.is_empty() {
        None
    } else {
        lines.reverse();
        Some(lines.join("\n"))
    }
}

/// If `fn_ast.body()` is None, try to extract the function body manually
/// by scanning the text after the param list / where clause and looking
/// for the first `{ ... }` or `= expr;`.
#[tracing::instrument(level = "trace", skip(fn_ast))]
fn fallback_extract_fn_body(fn_ast: &ast::Fn) -> Option<String> {
    let full_fn_text = fn_ast.syntax().text().to_string();
    trace!("fallback_extract_fn_body => raw fn text:\n{full_fn_text}");

    // 1) Find the offset after the parameter list / return type / where clause
    let mut start_search = 0usize;

    if let Some(plist) = fn_ast.param_list() {
        start_search = start_search.max(plist.syntax().text_range().end().into());
    }
    if let Some(ret) = fn_ast.ret_type() {
        start_search = start_search.max(ret.syntax().text_range().end().into());
    }
    if let Some(wc) = fn_ast.where_clause() {
        start_search = start_search.max(wc.syntax().text_range().end().into());
    }

    let tail = &full_fn_text.get(start_search..)?;
    trace!("fn body fallback: searching tail:\n{tail}");

    // 2) Attempt to find the first '{' and match braces
    if let Some(open_brace_pos) = tail.find('{') {
        let offset_in_full = start_search + open_brace_pos;
        let mut brace_stack = 0usize;
        let mut start_idx = None;
        let mut end_idx = None;

        for (i, ch) in full_fn_text.char_indices().skip(offset_in_full) {
            match ch {
                '{' => {
                    if brace_stack == 0 {
                        start_idx = Some(i);
                    }
                    brace_stack += 1;
                }
                '}' => {
                    brace_stack = brace_stack.saturating_sub(1);
                    if brace_stack == 0 {
                        end_idx = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let (Some(s), Some(e)) = (start_idx, end_idx) {
            return Some(full_fn_text[s..e].to_string());
        }
    }

    // 3) If no '{...}' block, check if this is `= expr;`
    if let Some(eq_pos) = tail.find('=') {
        let tail2 = &tail[eq_pos..];
        if let Some(semicol_pos) = tail2.find(';') {
            let snippet = &tail[(eq_pos)..(eq_pos + semicol_pos + 1)];
            // skip if it is "=>"
            if !snippet.contains("=>") {
                return Some(snippet.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod test_gather_fn_item {
    use super::*;

    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

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

    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new().with_docs().with_fn_bodies()
    }

    #[test]
    fn test_simple_fn_no_docs_no_attrs_with_body() {
        let snippet = r#"
            fn simple() {
            }
        "#;
        let root = parse_source(snippet);
        let fn_ast = find_first_fn(&root).expect("Expected a fn");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
        assert_eq!(*item.docs(), None);
        assert_eq!(*item.attributes(), None);
        assert!(item.body_source().is_some());
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        // (a) skip body if .with_fn_bodies_in_tests() is off
        let item_skip = gather_fn_item(&fn_ast, &opts_no_body_in_tests, &file_path, &crate_path);
        assert_eq!(*item_skip.body_source(), None);

        // (b) gather body if .with_fn_bodies_in_tests() is on
        let item_include = gather_fn_item(&fn_ast, &opts_with_test_bodies, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_node, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);
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

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let item = gather_fn_item(&fn_ast, &opts, &file_path, &crate_path);

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
