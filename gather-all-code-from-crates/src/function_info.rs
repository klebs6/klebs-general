crate::ix!();

/// Represents a function extracted from the AST.
#[derive(Builder,Setters,Getters,Debug, Clone)]
#[builder(setter(into))]
#[getset(get="pub",set="pub")]
pub struct FunctionInfo {
    /// The function's name (identifier)
    name: String,

    /// True if the function is public (`pub`)
    is_public: bool,

    /// True if the function is marked as a test (has `#[test]`)
    is_test: bool,

    /// List of attributes, e.g. `["#[test]", "#[cfg(...)]"]`
    attributes: Vec<String>,

    /// The complete function signature (e.g. "pub fn foo(x: i32) -> i32")
    signature: String,

    /// The function body (e.g. "{ x + 1 }"), or None if function is bodiless
    body: Option<String>,
}

/// Extracts functions from a syntax node, returning a list of `FunctionInfo`.
///
/// We find all `Fn` items in the file. For each:
/// - Extract name (skip if none).
/// - Check attributes for `#[test]`.
/// - Check visibility for `pub`.
/// - Extract signature line from the syntax.
/// - Extract body if present.
pub fn extract_functions_from_ast(syntax: &SyntaxNode, remove_doc_comments: bool) -> Vec<FunctionInfo> {
    let mut results = Vec::new();

    for node in syntax.descendants() {
        if let Some(fn_def) = ast::Fn::cast(node.clone()) {
            // Get function name
            let name_node = match fn_def.name() {
                Some(n) => n.text().to_string(),
                None => continue, // skip functions without names
            };

            let mut attributes = Vec::new();
            let mut is_test = false;

            for attr in fn_def.attrs() {
                let txt = attr.syntax().text().to_string();
                attributes.push(txt.trim().to_string());
                if txt.contains("#[test]") {
                    is_test = true;
                }
            }

            let is_public = fn_def.visibility().map_or(false, |v| v.syntax().text().to_string().contains("pub"));

            // Extract signature:
            let signature = {
                let start = fn_def.syntax().text_range().start();
                let mut end = start;
                let mut found_end = false;
                for token in fn_def.syntax().descendants_with_tokens() {
                    if let Some(t) = token.as_token() {
                        if t.kind() == ra_ap_syntax::SyntaxKind::L_CURLY 
                            || t.kind() == ra_ap_syntax::SyntaxKind::SEMICOLON {
                            end = t.text_range().start();
                            found_end = true;
                            break;
                        }
                    }
                }
                if !found_end {
                    // If we never found a '{' or ';', just take the whole function node text.
                    fn_def.syntax().text().to_string()
                } else {
                    // slice the text up to `end`
                    let text = fn_def.syntax().text().to_string();
                    let start_idx = 0;
                    let end_idx = (end - start).into();
                    if end_idx <= text.len() {
                        text[..end_idx].trim_end().to_string()
                    } else {
                        text // fallback: entire text if indices mismatch
                    }
                }
            };

            // Extract body if present
            let body = fn_def.body().map(|b| b.syntax().text().to_string());

            if remove_doc_comments {

                // After extracting the `signature` text
                let mut signature = signature.trim_end().to_string();

                // Remove doc comment lines from the signature
                let mut filtered_lines = Vec::new();
                for line in signature.lines() {
                    let trimmed = line.trim_start();
                    // If this line is a doc comment, skip it
                    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                        continue;
                    }
                    filtered_lines.push(line);
                }

                // Rejoin lines without doc comments
                signature = filtered_lines.join("\n");

                // Now signature should be clean of doc comments
                results.push(FunctionInfo {
                    name: name_node,
                    is_public,
                    is_test,
                    attributes,
                    signature,
                    body,
                });

            } else {

                results.push(FunctionInfo {
                    name: name_node,
                    is_public,
                    is_test,
                    attributes,
                    signature: signature.trim_end().to_string(),
                    body,
                });

            }
        }
    }

    results
}

#[cfg(test)]
mod function_info_tests {
    use super::*;

    fn parse_source_code(code: &str) -> SyntaxNode {
        let parsed = SourceFile::parse(code, Edition::CURRENT);
        assert!(parsed.errors().is_empty(), "Parsing errors: {:?}", parsed.errors());
        parsed.tree().syntax().clone()
    }

    #[test]
    fn test_extract_functions_from_ast_basic() {
        let code = r#"
            fn foo() { println!("hello"); }
            pub fn bar(x: i32) -> i32 { x + 1 }
            #[test]
            fn test_me() {}
            fn unnamed() {}
        "#;
        let syntax = parse_source_code(code);
        let funcs = extract_functions_from_ast(&syntax, false);

        // unnamed function should be skipped because it has no name node
        // (The above code actually provides a name node: `unnamed`.)
        // If we want it truly unnamed, we'd have to omit the function name entirely.
        // But since it's named `unnamed`, it will be recognized.
        // To simulate a truly unnamed function, we must provide invalid syntax, which we cannot do if we want no parse errors.
        // Instead, let's assume we just won't find a name-less function and rely on the fact we only want 3 results total.
        //
        // Currently, we have 4 named functions: foo, bar, test_me, unnamed.
        // To skip one, we need a function without a name. That's not valid Rust syntax.
        // Instead, let's just assert we got all 4 and then decide which we consider correct.

        assert_eq!(funcs.len(), 4);

        let foo = funcs.iter().find(|f| f.name == "foo").unwrap();
        assert!(!foo.is_public);
        assert!(!foo.is_test);
        assert!(foo.signature.contains("fn foo("));
        assert!(foo.body.as_ref().unwrap().contains("println"));

        let bar = funcs.iter().find(|f| f.name == "bar").unwrap();
        assert!(bar.is_public);
        assert!(!bar.is_test);
        assert!(bar.signature.contains("pub fn bar(x: i32) -> i32"));
        assert!(bar.body.as_ref().unwrap().contains("x + 1"));

        let test_me = funcs.iter().find(|f| f.name == "test_me").unwrap();
        assert!(!test_me.is_public);
        assert!(test_me.is_test);
        assert!(test_me.attributes.iter().any(|a| a.contains("#[test]")));

        let unnamed = funcs.iter().find(|f| f.name == "unnamed").unwrap();
        // `unnamed` is actually named "unnamed", so it's valid. If the goal was to skip it,
        // we need to modify the code snippet so that it's invalid and doesn't produce a name.
        // For now, let's accept it as a recognized function.
        assert!(!unnamed.is_public);
        assert!(!unnamed.is_test);
    }

    #[test]
    fn test_extract_functions_from_ast_no_body() {
        let code = r#"
            extern "C" {
                pub fn interface();
            }
        "#;
        let syntax = parse_source_code(code);
        let funcs = extract_functions_from_ast(&syntax, false);

        // We now have a single, bodyless function declared in an extern block.
        // This should be recognized as a function with no body.
        assert_eq!(funcs.len(), 1);
        let interface = &funcs[0];
        assert!(interface.is_public);
        assert!(interface.body.is_none());
        println!("interface.signature: {:#?}", interface.signature);
        assert!(interface.signature.contains("pub fn interface()"));
    }
}
