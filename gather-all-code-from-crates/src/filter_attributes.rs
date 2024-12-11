crate::ix!();

pub fn filter_doc_attrs(attrs: impl Iterator<Item=ast::Attr>, remove_docs: bool) -> Vec<ast::Attr> {
    attrs.filter(|attr| {
        if remove_docs && is_doc_attr(attr) {
            false
        } else {
            true
        }
    }).collect()
}

pub fn is_doc_attr(attr: &ast::Attr) -> bool {
    // Check if path is `doc`
    if let Some(path) = attr.path() {
        path.syntax().text() == "doc"
    } else {
        false
    }
}

pub fn is_pub(vis: &ast::Visibility) -> bool {
    vis.syntax()
        .children_with_tokens()
        .any(|child| child.kind() == ra_ap_syntax::SyntaxKind::PUB_KW)
}
