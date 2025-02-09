// ---------------- [ File: workspacer-syntax/src/public.rs ]
crate::ix!();

pub fn is_node_public(node: &SyntaxNode) -> bool {
    // Inner function to check visibility
    let has_visibility = |node: &SyntaxNode| {
        node.children()
            .find_map(ast::Visibility::cast)
            .map_or(false, |vis| matches!(
                vis.kind(), 
                ast::VisibilityKind::Pub | ast::VisibilityKind::PubCrate | ast::VisibilityKind::PubSuper | ast::VisibilityKind::PubSelf
            ))
    };

    let kind = node.kind();
    let is_public = match kind {
        SyntaxKind::FN => has_visibility(node),
        SyntaxKind::STRUCT => has_visibility(node),
        SyntaxKind::ENUM => has_visibility(node),
        SyntaxKind::TRAIT => has_visibility(node),
        SyntaxKind::TYPE_ALIAS => has_visibility(node),
        SyntaxKind::MACRO_RULES => {
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
        },
        _ => false,
    };

    println!("Node kind: {:?}, Is public: {}", kind, is_public);
    is_public
}

