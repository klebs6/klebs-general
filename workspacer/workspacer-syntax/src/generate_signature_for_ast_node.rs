// ---------------- [ File: src/generate_signature_for_ast_node.rs ]
crate::ix!();

/// A trait to generate a "signature" string (or declaration line)
/// for different AST nodes like `Fn`, `Struct`, `Enum`, etc.
pub trait GenerateSignature: fmt::Debug + Clone {
    /// Generate a textual signature, optionally embedding doc lines
    /// (passed in from your doc-extraction routine).
    fn generate_signature(&self) -> String;
}

#[cfg(test)]
mod test_generate_signature_robustness {
    use super::*;
    use ra_ap_syntax::{SourceFile, AstNode, Edition};

    /// Helper: parse a snippet of code, return the first node of type T we find.
    fn parse_first_node_of_type<T: AstNode>(code: &str) -> T {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .descendants()
            .find_map(T::cast)
            .expect("Should parse and find a node of desired AST type.")
    }

    /// Helper: unify doc lines as a single string, as though we extracted them from the AST
    fn docs_from_lines(lines: &[&str]) -> String {
        lines.join("\n")
    }

    // -------------------------------- Fn Tests --------------------------------

    #[traced_test]
    fn test_fn_signature_no_params_no_return() {
        let code = r#"
            pub fn simple_fn() {}
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature();
        assert!(signature.contains("pub fn simple_fn()"), "Signature: {signature}");
        assert!(signature.contains("{ /* ... */ }"), "Should have curly placeholder body");
    }

    #[traced_test]
    fn test_fn_signature_with_params_and_return() {
        let code = r#"
            pub fn add(a: i32, b: i32) -> i32 { a + b }
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature();
        assert!(signature.contains("pub fn add(a: i32, b: i32) -> i32"), "Signature: {signature}");
    }

    #[traced_test]
    fn test_fn_signature_with_generics_where_clause() {
        let code = r#"
            pub fn generic_fn<T: Clone>(x: T) -> T where T: std::fmt::Debug {
                x
            }
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature();
        assert!(
            signature.contains("pub fn generic_fn<T: Clone>(x: T) -> T where T: std::fmt::Debug"),
            "Signature: {signature}"
        );
    }

    #[traced_test]
    fn test_fn_signature_with_docs() {
        let code = r#"
            /// This function does something.
            /// Another line of docs.
            pub fn documented() {}
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let doc_text = docs_from_lines(&[
            "This function does something.",
            "Another line of docs."
        ]);
        let signature = fn_node.generate_signature();
        assert!(signature.contains("/// This function does something."));
        assert!(signature.contains("/// Another line of docs."));
        assert!(signature.contains("pub fn documented()"));
    }

    // -------------------------------- Struct Tests --------------------------------

    #[traced_test]
    fn test_struct_signature_no_generics() {
        let code = r#"
            pub struct MyStruct { x: i32 }
        "#;
        let st_node: ast::Struct = parse_first_node_of_type(code);

        let signature = st_node.generate_signature();
        assert!(signature.contains("pub struct MyStruct"), "Signature: {signature}");
        assert!(signature.contains("{ /* fields omitted */ }"), "Signature: {signature}");
    }

    #[traced_test]
    fn test_struct_signature_with_generics_and_docs() {
        let code = r#"
            /// A generic struct
            pub struct Container<T> where T: Clone {
                value: T
            }
        "#;
        let st_node: ast::Struct = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["A generic struct"]);

        let signature = st_node.generate_signature();
        assert!(signature.contains("/// A generic struct"));
        assert!(signature.contains("pub struct Container<T> where T: Clone"));
    }

    // -------------------------------- Enum Tests --------------------------------

    #[traced_test]
    fn test_enum_signature_with_generics_where_clause() {
        let code = r#"
            pub enum MyEnum<T> where T: Copy {
                A(T),
                B
            }
        "#;
        let enum_node: ast::Enum = parse_first_node_of_type(code);

        let signature = enum_node.generate_signature();
        assert!(signature.contains("pub enum MyEnum<T> where T: Copy"), "Signature: {signature}");
    }

    // -------------------------------- Trait Tests --------------------------------

    #[traced_test]
    fn test_trait_signature() {
        let code = r#"
            pub trait MyTrait {
                fn required_method(&self);
            }
        "#;
        let trait_node: ast::Trait = parse_first_node_of_type(code);

        let signature = trait_node.generate_signature();
        assert!(signature.contains("pub trait MyTrait"));
        assert!(signature.contains("{ /* items omitted */ }"));
    }

    #[traced_test]
    fn test_trait_signature_with_generics_where_clause_and_docs() {
        let code = r#"
            /// This trait does stuff
            pub trait DoStuff<T> where T: Clone {
                fn do_it(&self, x: T);
            }
        "#;
        let trait_node: ast::Trait = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["This trait does stuff"]);

        let signature = trait_node.generate_signature();
        assert!(signature.contains("/// This trait does stuff"));
        assert!(signature.contains("pub trait DoStuff<T> where T: Clone"));
    }

    // -------------------------------- TypeAlias Tests --------------------------------

    #[traced_test]
    fn test_type_alias_signature() {
        let code = r#"
            pub type MyAlias = i32;
        "#;
        let type_node: ast::TypeAlias = parse_first_node_of_type(code);

        let signature = type_node.generate_signature();
        assert!(signature.contains("pub type MyAlias"));
        assert!(signature.contains("= /* aliased type omitted */;"));
    }

    #[traced_test]
    fn test_type_alias_signature_with_generics_where() {
        let code = r#"
            pub type MyGenericAlias<T> where T: Default = Vec<T>;
        "#;
        let type_node: ast::TypeAlias = parse_first_node_of_type(code);

        let signature = type_node.generate_signature();
        assert!(
            signature.contains("pub type MyGenericAlias<T> where T: Default = /* aliased type omitted */;"),
            "Signature: {signature}"
        );
    }

    // -------------------------------- MacroRules Tests --------------------------------

    #[traced_test]
    fn test_macro_rules_signature() {
        let code = r#"
            #[macro_export]
            macro_rules! my_macro {
                () => {};
            }
        "#;
        let mac_node: ast::MacroRules = parse_first_node_of_type(code);

        let signature = mac_node.generate_signature();
        assert!(signature.contains("macro_rules! my_macro"));
        assert!(signature.contains("{ /* macro body omitted */ }"));
    }

    #[traced_test]
    fn test_macro_rules_signature_with_docs() {
        let code = r#"
            /// A fancy macro
            #[macro_export]
            macro_rules! fancy_macro {
                () => {};
            }
        "#;
        let mac_node: ast::MacroRules = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["A fancy macro"]);

        let signature = mac_node.generate_signature();
        assert!(signature.contains("/// A fancy macro"));
        assert!(signature.contains("macro_rules! fancy_macro"));
    }
}
