// ---------------- [ File: workspacer-syntax/src/public.rs ]
crate::ix!();

/// Check if a node is “public,” e.g. `pub fn`, `pub struct`, or `#[macro_export]`.
pub fn is_node_public(node: &SyntaxNode) -> bool {
    let has_visibility = || {
        node.children()
            .find_map(ast::Visibility::cast)
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
            if let Some(macro_node) = ast::MacroRules::cast(node.clone()) {
                for attr in macro_node.attrs() {
                    if let Some(meta) = attr.meta() {
                        if let Some(path_node) = meta.path() {
                            if path_node.syntax().text().to_string() == "macro_export" {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        _ => false,
    };

    debug!("Node kind: {:?}, is_public: {}", kind, is_public);
    is_public
}

#[cfg(test)]
mod test_is_node_public_exhaustive {
    use super::*;

    /// Helper to parse code, then return the first SyntaxNode that isn't trivial.
    fn parse_first_non_trivial_node(code: &str) -> SyntaxNode {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .children()
            .find(|child| child.kind() != SyntaxKind::WHITESPACE && child.kind() != SyntaxKind::COMMENT)
            .unwrap_or_else(|| panic!("Could not find a non-trivial node in:\n{code}"))
    }

    #[traced_test]
    fn test_public_fn_is_node_public() {
        info!("Testing pub fn is recognized as public.");
        let code = r#"pub fn do_something() {}"#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(is_node_public(&node));
    }

    #[traced_test]
    fn test_private_fn_is_node_public() {
        info!("Testing private fn is recognized as not public.");
        let code = r#"fn do_something() {}"#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::FN);
        assert!(!is_node_public(&node));
    }

    #[traced_test]
    fn test_pub_struct_is_node_public() {
        info!("Testing pub struct is recognized as public.");
        let code = r#"
            pub struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::STRUCT);
        assert!(is_node_public(&node));
    }

    #[traced_test]
    fn test_private_struct_is_node_public() {
        info!("Testing private struct is recognized as not public.");
        let code = r#"
            struct MyStruct {
                field: i32
            }
        "#;
        let node = parse_first_non_trivial_node(code);
        assert_eq!(node.kind(), SyntaxKind::STRUCT);
        assert!(!is_node_public(&node));
    }

    #[traced_test]
    fn test_macro_rules_with_macro_export_is_node_public() {
        info!("Testing macro_rules! with #[macro_export] is recognized as public.");
        let code = r#"
            #[macro_export]
            macro_rules! my_macro {
                () => {}
            }
        "#;
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        let macro_node = syntax.descendants().find(|n| n.kind() == SyntaxKind::MACRO_RULES).unwrap();
        assert!(is_node_public(&macro_node));
    }

    #[traced_test]
    fn test_macro_rules_without_macro_export_is_node_public() {
        info!("Testing macro_rules! without macro_export is recognized as private.");
        let code = r#"
            macro_rules! my_macro {
                () => {}
            }
        "#;
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        let macro_node = syntax.descendants().find(|n| n.kind() == SyntaxKind::MACRO_RULES).unwrap();
        assert!(!is_node_public(&macro_node));
    }
}
