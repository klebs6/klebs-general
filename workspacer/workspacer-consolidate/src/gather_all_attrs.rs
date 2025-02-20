// ---------------- [ File: src/gather_all_attrs.rs ]
crate::ix!();

/// Gather all the raw attributes (e.g. `#[derive(Debug)]`, `#[cfg(feature="xyz")]`, etc.)
/// into a single string, one per line. Returns `None` if no attributes found.
pub fn gather_all_attrs(node: &SyntaxNode) -> Option<String> {
    use ra_ap_syntax::ast::Attr;

    let mut lines = Vec::new();
    for child_attr in node.children().filter_map(Attr::cast) {
        // Easiest is to grab the exact text:
        let txt = child_attr.syntax().text().to_string();
        lines.push(txt);
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}
