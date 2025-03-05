// ---------------- [ File: workspacer-syntax/src/generate_signature_for_ast_node.rs ]
crate::ix!();

/// A trait to generate a textual “signature” (or declaration line(s))
/// for different AST nodes (Fn, Struct, Enum, etc.), possibly with doc lines, etc.
pub trait GenerateSignature: fmt::Debug + Clone {
    /// Flexible entry point: generate signature with the given `options`.
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String;

    /// Default convenience wrapper: generate signature with fully expanded detail & doc lines.
    fn generate_signature(&self) -> String {
        let default_opts = SignatureOptions::default();
        self.generate_signature_with_opts(&default_opts)
    }
}

#[cfg(test)]
mod test_generate_signature_robustness {
    use super::*;

    /// Helper: parse a snippet of code, return the first node of type T we find.
    fn parse_first_node_of_type<T: AstNode>(code: &str) -> T {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .descendants()
            .find_map(T::cast)
            .expect("Should parse and find a node of desired AST type.")
    }

    #[traced_test]
    fn test_fully_expanded_fn() {
        info!("Testing a function signature in fully expanded mode with doc lines included.");
        let code = r#"
            /// A doc line
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let fn_node: ast::Fn = parse_first_node_of_type(code);
        let opts = SignatureOptionsBuilder::default()
            .fully_expand(true)
            .include_docs(true)
            .build()
            .unwrap();

        let signature = fn_node.generate_signature_with_opts(&opts);
        debug!(?signature, "Resulting signature");
        assert!(signature.contains("/// A doc line"));
        assert!(signature.contains("pub fn add(a: i32, b: i32) -> i32"));
    }

    #[traced_test]
    fn test_minimal_fn() {
        info!("Testing a function signature in minimal mode (no doc lines, placeholders).");
        let code = r#"
            /// Something
            pub async fn do_stuff(x: &str) {}
        "#;

        let fn_node: ast::Fn = parse_first_node_of_type(code);
        let opts = SignatureOptionsBuilder::default()
            .fully_expand(false)
            .include_docs(false)
            .build()
            .unwrap();

        let signature = fn_node.generate_signature_with_opts(&opts);
        debug!(?signature, "Resulting signature");
        // We wouldn't see doc lines or expansions, but let's see if it's minimal.
        assert!(!signature.contains("/// Something"));
        assert!(signature.contains("pub async fn do_stuff(x: &str)")); 
    }
}
