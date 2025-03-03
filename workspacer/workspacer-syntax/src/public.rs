// ---------------- [ File: src/public.rs ]
crate::ix!();

// src/public.rs (or wherever you have is_node_public):
//
pub fn is_node_public(node: &SyntaxNode) -> bool {
    // This helper checks if the node literally has a `Visibility` child spelled `pub`,
    // e.g. `pub fn something()`.
    let has_visibility = || {
        node.children()
            .find_map(ast::Visibility::cast)
            // If we find a Visibility node, check if it starts with "pub"
            // or "pub(" or any recognized variant.
            .map_or(false, |vis| {
                let text = vis.syntax().text().to_string();
                text.starts_with("pub")
            })
    };

    let kind = node.kind();
    let is_public = match kind {
        SyntaxKind::FN
        | SyntaxKind::STRUCT
        | SyntaxKind::ENUM
        | SyntaxKind::TRAIT
        | SyntaxKind::TYPE_ALIAS => has_visibility(),
        
        SyntaxKind::MACRO_RULES => {
            // special check if there's an attribute like #[macro_export]
            use ra_ap_syntax::ast::HasAttrs;
            ast::MacroRules::cast(node.clone()).map_or(false, |macro_node| {
                for attr in macro_node.attrs() {
                    if let Some(meta) = attr.meta() {
                        if let Some(path_node) = meta.path() {
                            if path_node.syntax().text().to_string() == "macro_export" {
                                return true;
                            }
                        }
                    }
                }
                false
            })

        }

        // Everything else is not considered "public"
        _ => false,
    };

    debug!("Node kind: {:?}, is_public: {}", kind, is_public);
    is_public
}

#[cfg(test)]
mod test_is_node_public_exhaustive {
    use super::*;

    /// Helper that parses code, then returns the first SyntaxNode that isn't a top-level Token.
    /// We expect the first node to be a function/item that we want to test.
    fn parse_first_non_trivial_node(code: &str) -> SyntaxNode {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        // Find a child that is not a TOKEN, e.g. FN, STRUCT, ENUM, etc.
        syntax
            .children()
            .find(|child| child.kind() != SyntaxKind::WHITESPACE)
            .unwrap_or_else(|| panic!("Could not find any non-whitespace node in the test snippet."))
    }

    /// If we need to grab a specific node kind from within a snippet (e.g. for macro tests),
    /// use this helper to find the first node matching the provided kind.
    fn parse_first_node_of_kind(code: &str, kind: SyntaxKind) -> SyntaxNode {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .descendants()
            .find(|n| n.kind() == kind)
            .unwrap_or_else(|| panic!("Did not find a node of kind {:?} in snippet:\n{}", kind, code))
    }

    #[traced_test]
    fn test_public_fn_is_node_public() {
        let code = r#"
            pub fn do_something() {}
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN, "Should be an Fn node.");
        assert!(is_node_public(&node), "Expected pub fn to be public.");
    }

    #[traced_test]
    fn test_crate_visibility_fn_is_node_public() {
        let code = r#"
            pub(crate) fn do_something() {}
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(is_node_public(&node), "Expected pub(crate) fn to be public by the function's logic.");
    }

    #[traced_test]
    fn test_fn_pub_super_is_node_public() {
        let code = r#"
            pub(super) fn do_something() {}
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(is_node_public(&node), "Expected pub(super) fn to be recognized as public.");
    }

    #[traced_test]
    fn test_fn_pub_self_is_node_public() {
        let code = r#"
            pub(self) fn do_something() {}
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(is_node_public(&node), "Expected pub(self) fn to be recognized as public.");
    }

    #[traced_test]
    fn test_private_fn_is_node_public() {
        let code = r#"
            fn do_something() {}
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(!is_node_public(&node), "Expected non-pub fn to be private.");
    }

    #[traced_test]
    fn test_pub_struct_is_node_public() {
        let code = r#"
            pub struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::STRUCT);
        assert!(is_node_public(&node), "Expected pub struct to be public.");
    }

    #[traced_test]
    fn test_struct_pub_crate_is_node_public() {
        let code = r#"
            pub(crate) struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::STRUCT);
        assert!(is_node_public(&node), "Expected pub(crate) struct to be recognized as public.");
    }

    #[traced_test]
    fn test_private_struct_is_node_public() {
        let code = r#"
            struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::STRUCT);
        assert!(!is_node_public(&node), "Expected private struct (no pub) to be private.");
    }

    #[traced_test]
    fn test_pub_enum_is_node_public() {
        let code = r#"
            pub enum MyEnum {
                A,
                B
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::ENUM);
        assert!(is_node_public(&node), "Expected pub enum to be public.");
    }

    #[traced_test]
    fn test_private_enum_is_node_public() {
        let code = r#"
            enum MyEnum {
                A,
                B
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::ENUM);
        assert!(!is_node_public(&node), "Expected private enum to be private.");
    }

    #[traced_test]
    fn test_pub_trait_is_node_public() {
        let code = r#"
            pub trait MyTrait {
                fn something(&self);
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::TRAIT);
        assert!(is_node_public(&node), "Expected pub trait to be public.");
    }

    #[traced_test]
    fn test_private_trait_is_node_public() {
        let code = r#"
            trait MyTrait {
                fn something(&self);
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::TRAIT);
        assert!(!is_node_public(&node), "Expected trait with no visibility to be private.");
    }

    #[traced_test]
    fn test_pub_type_alias_is_node_public() {
        let code = r#"
            pub type MyAlias = i32;
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::TYPE_ALIAS);
        assert!(is_node_public(&node), "Expected pub type alias to be public.");
    }

    #[traced_test]
    fn test_private_type_alias_is_node_public() {
        let code = r#"
            type MyAlias = i32;
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::TYPE_ALIAS);
        assert!(!is_node_public(&node), "Expected private type alias to be private.");
    }

    #[traced_test]
    fn test_macro_rules_with_macro_export_is_node_public() {
        let code = r#"
            #[macro_export]
            macro_rules! my_macro {
                () => {};
            }
        "#;
        // We specifically want the MacroRules node, not the attribute.
        let node = parse_first_node_of_kind(code, SyntaxKind::MACRO_RULES);
        assert_eq!(node.kind(), SyntaxKind::MACRO_RULES);
        assert!(is_node_public(&node), "Expected macro_rules! with #[macro_export] to be public.");
    }

    #[traced_test]
    fn test_macro_rules_without_macro_export_is_node_public() {
        let code = r#"
            // no macro_export
            macro_rules! my_macro {
                () => {};
            }
        "#;
        let node = parse_first_node_of_kind(code, SyntaxKind::MACRO_RULES);
        assert_eq!(node.kind(), SyntaxKind::MACRO_RULES);
        assert!(!is_node_public(&node), "Expected macro_rules! without #[macro_export] to be private.");
    }

    #[traced_test]
    fn test_unrelated_code_snippet_is_node_public() {
        let code = r#"
            let x = 123; // just a random statement
        "#;
        // There's no top-level item here; parse_first_non_trivial_node will
        // return the "let x = 123" node, which has kind LOCAL. 
        let node = parse_first_non_trivial_node(code);
        //assert_eq!(node.kind(), SyntaxKind::LOCAL);
        assert!(!is_node_public(&node), "Local variable declarations should be treated as private/unrelated.");
    }

    #[traced_test]
    fn test_all_visibility_kinds_fn_are_public() {
        // Test pub, pub(crate), pub(super), pub(self)
        let code = r#"
            pub fn f1() {}
            pub(crate) fn f2() {}
            pub(super) fn f3() {}
            pub(self) fn f4() {}
        "#;
        // We can collect all child items that are function nodes:
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        let fn_nodes: Vec<SyntaxNode> = syntax
            .descendants()
            .filter(|n| n.kind() == SyntaxKind::FN)
            .collect();

        assert_eq!(fn_nodes.len(), 4, "Should parse four function items.");

        for (idx, fn_node) in fn_nodes.iter().enumerate() {
            assert!(is_node_public(fn_node), "Function #{} in the snippet should be public.", idx + 1);
        }
    }
}
