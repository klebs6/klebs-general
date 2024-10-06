crate::ix!();

/// Determines if a given syntax node in a Rust AST represents a public entity.
///
/// This function checks the visibility of various kinds of syntax nodes
/// that can be part of a Rust Abstract Syntax Tree (AST), and returns
/// `true` if the node is marked as public (`pub`), or `false` otherwise.
///
/// Currently, this function handles the following kinds of syntax nodes:
/// - Functions
/// - Structs
/// - Enums
/// - Traits
/// - Type Aliases
///
/// # Arguments
///
/// - `node`: A reference to the `SyntaxNode` being checked for visibility.
///
/// # Returns
///
/// Returns `true` if the syntax node is marked as public, `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use scan_crate_for_typedefs::is_node_public;
///
/// use ra_ap_syntax::SyntaxNode; // Please replace with the actual import
///
/// let node = todo!();/* Obtain the SyntaxNode instance somehow */;
///
/// let is_public = is_node_public(&node);
/// ```
///
/// # Note
///
/// More syntax node kinds can be added in the future to extend the function's functionality.
pub fn is_node_public(node: &SyntaxNode) -> bool {

    // Inner function to check visibility
    let has_visibility = |node: &SyntaxNode| {

        node.children()

            .find_map(ast::Visibility::cast)

            .map_or(false, |vis| matches!(
                vis.kind(), 
                ast::VisibilityKind::Pub
                // Uncomment these lines if you want to handle these visibilities
                //| ast::VisibilityKind::PubCrate
                //| ast::VisibilityKind::PubSuper
                //| ast::VisibilityKind::PubSelf
            ))
    };

    // Check the syntax node kind and then its visibility
    if let Some(_) = ast::Fn::cast(node.clone()) {
        has_visibility(node)

    } else if let Some(_) = ast::Struct::cast(node.clone()) {
        has_visibility(node)

    } else if let Some(_) = ast::Enum::cast(node.clone()) {
        has_visibility(node)

    } else if let Some(_) = ast::Trait::cast(node.clone()) {
        has_visibility(node)

    } else if let Some(_) = ast::TypeAlias::cast(node.clone()) {
        has_visibility(node)

    } else if let Some(macro_node) = ast::MacroRules::cast(node.clone()) {

        use ra_ap_syntax::ast::HasAttrs;

        for attr in macro_node.attrs() {

            if let Some(meta) = attr.meta() {

                if let Some(path_node) = meta.path() {

                    let path_string = path_node.syntax().text().to_string();

                    if path_string.as_str() == "macro_export" {
                        return true;
                    }
                }
            }
        }

        false

    } else {

        // Add more cases for other AST node kinds here
        false
    }
}
