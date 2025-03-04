// ---------------- [ File: src/fallback_scan_node_text.rs ]
crate::ix!();

pub fn fallback_scan_node_text(node: &SyntaxNode) -> Vec<String> {
    info!(
        "fallback_scan_node_text => start; node.kind()={:?}",
        node.kind()
    );
    let node_text = node.text().to_string();
    trace!("node_text length={}", node_text.len());

    let mut results = Vec::new();
    for line in node_text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            let line = if trimmed.ends_with('\n') {
                trimmed.to_string()
            } else {
                format!("{}\n", trimmed)
            };
            trace!("Found line-comment => {:?}", line);
            results.push(line);
        } else if trimmed.is_empty() {
            warn!("Blank line => stopping fallback scan");
            break;
        } else {
            debug!("Non-comment line => stopping fallback scan");
            break;
        }
    }

    debug!("fallback_scan_node_text => returning {} lines", results.len());
    info!("fallback_scan_node_text => done");
    results
}

#[cfg(test)]
mod test_fallback_scan_node_text {
    use super::*;
    use ra_ap_syntax::SourceFile;

    fn parse_node(src: &str) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    #[test]
    fn test_no_comments() {
        let node = parse_node("fn main() {}");
        let gathered = fallback_scan_node_text(&node);
        assert!(gathered.is_empty(), "No // lines => empty");
    }

    #[test]
    fn test_consecutive_comments() {
        let node = parse_node("// line1\n// line2\nfn main(){}");
        let gathered = fallback_scan_node_text(&node);
        assert_eq!(gathered.len(), 2);
        assert_eq!(gathered[0], "// line1\n");
        assert_eq!(gathered[1], "// line2\n");
    }

    #[test]
    fn test_stops_on_blank_line() {
        let node = parse_node("// comment\n\nfn main(){}");
        let gathered = fallback_scan_node_text(&node);
        assert_eq!(gathered.len(), 1, "Should stop at blank line");
    }

    #[test]
    fn test_stops_on_non_comment() {
        let node = parse_node("// c\nfn main(){}");
        let gathered = fallback_scan_node_text(&node);
        assert_eq!(gathered.len(), 1, "Should stop at 'fn'");
    }
}
