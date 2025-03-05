// ---------------- [ File: workspacer-consolidate/src/is_in_test_module.rs ]
crate::ix!();

/// Returns true if any ancestor is a `mod` that has `#[cfg(test)]`.
pub fn is_in_test_module(mut node: SyntaxNode) -> bool {
    use ra_ap_syntax::SyntaxKind::{MODULE, SOURCE_FILE};
    while node.kind() != SOURCE_FILE {
        if node.kind() == MODULE {
            if has_cfg_test_attr(&node) {
                return true;
            }
        }
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            break;
        }
    }
    false
}

#[cfg(test)]
mod test_is_in_test_module {
    use super::*;

    /// Helper to parse a Rust snippet into a `SyntaxNode`.
    fn parse_rust_snippet(snippet: &str) -> SyntaxNode {
        // In `ra_ap_syntax`, you'd typically do something like:
        let parse = SourceFile::parse(snippet,Edition::Edition2024);
        parse.tree().syntax().clone()
    }

    /// Helper to find the first sub-node with kind MODULE in a syntax tree.
    /// We'll do a simplistic approach: walk children recursively.
    fn find_first_module_node(root: &SyntaxNode) -> Option<SyntaxNode> {
        for child in root.descendants() {
            if child.kind() == SyntaxKind::MODULE {
                return Some(child);
            }
        }
        None
    }

    /// 1) If there's no `#[cfg(test)]` attribute at all, is_in_test_module should return false.
    #[test]
    fn test_is_in_test_module_none() {
        let code = r#"
            mod normal_mod {
                fn some_function() {}
            }
        "#;
        let root = parse_rust_snippet(code);
        let module_node = find_first_module_node(&root).expect("Expected a mod");
        let result = is_in_test_module(module_node);
        assert_eq!(result, false, "No #[cfg(test)] => not in test module");
    }

    /// 2) A single module with `#[cfg(test)]` => is_in_test_module should return true.
    #[test]
    fn test_is_in_test_module_single_cfg_test() {
        let code = r#"
            #[cfg(test)]
            mod test_mod {
                fn test_fn() {}
            }
        "#;
        let root = parse_rust_snippet(code);
        let module_node = find_first_module_node(&root).expect("Expected a mod");
        let result = is_in_test_module(module_node);
        assert_eq!(result, true, "#[cfg(test)] => is in test module");
    }

    /// 3) A nested mod structure: only the outer mod has `#[cfg(test)]`.
    ///    The inner mod inherits that from an ancestor, so is_in_test_module should be true.
    #[test]
    fn test_is_in_test_module_nested_inherits_cfg_test() {
        let code = r#"
            #[cfg(test)]
            mod outer {
                mod inner {
                    fn nested_fn() {}
                }
            }
        "#;
        let root = parse_rust_snippet(code);
        // The `inner` mod is the second MODULE node
        let mut modules = root.descendants().filter(|n| n.kind() == SyntaxKind::MODULE);
        let outer_node = modules.next().unwrap();
        let inner_node = modules.next().unwrap();
        assert!(is_in_test_module(outer_node.clone()), "outer has cfg(test)");
        assert!(is_in_test_module(inner_node.clone()), "inner inherits from outer");
    }

    /// 4) A nested mod structure: the outer doesn't have `#[cfg(test)]`, but the inner one does.
    #[test]
    fn test_is_in_test_module_nested_inner_has_cfg_test() {
        let code = r#"
            mod outer {
                #[cfg(test)]
                mod inner {
                    fn only_tests_in_here() {}
                }
            }
        "#;
        let root = parse_rust_snippet(code);
        let mut modules = root.descendants().filter(|n| n.kind() == SyntaxKind::MODULE);
        let outer_node = modules.next().unwrap();
        let inner_node = modules.next().unwrap();
        assert!(!is_in_test_module(outer_node), "outer has no cfg(test)");
        assert!(is_in_test_module(inner_node), "inner has cfg(test)");
    }

    /// 5) Source file root node doesn't have a parent, so once we reach SOURCE_FILE, we stop => false if no mod with cfg(test).
    #[test]
    fn test_is_in_test_module_reaches_source_file() {
        let code = r#"
            fn something() {}
        "#;
        let root = parse_rust_snippet(code);
        // There's no module in this snippet, so let's just pass the root as is
        let result = is_in_test_module(root.clone());
        assert_eq!(result, false, "No mod at all => definitely not in test module");
    }
}
