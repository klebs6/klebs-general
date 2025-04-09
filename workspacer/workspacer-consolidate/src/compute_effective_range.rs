// ---------------- [ File: workspacer-consolidate/src/compute_effective_range.rs ]
crate::ix!();

/// Returns a `TextRange` for `node` excluding any leading/trailing
/// **normal** comments and whitespace. Doc comments (///, //!,
/// /**, /*! etc.) remain inside. Normal `//` or `/* ... */` is trimmed.
pub fn compute_effective_range(node: &SyntaxNode) -> TextRange {
    let raw = node.text_range();
    if raw.is_empty() {
        return raw;
    }

    // Gather all tokens belonging to this node
    let tokens: Vec<SyntaxToken> = node
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .collect();

    let mut start_offset = raw.start();
    let mut end_offset   = raw.end();

    // 1) Move forward from the front while each token is normal (non‐doc) comment or whitespace
    for t in &tokens {
        if t.text_range().start() < start_offset || t.text_range().end() > end_offset {
            continue; // outside node’s raw range
        }
        if is_whitespace_or_normal_comment(t) {
            // Move our start forward past this token
            let next_start = t.text_range().end();
            if next_start < end_offset {
                start_offset = next_start;
            }
        } else {
            // Found something not normal‐comment => stop
            break;
        }
    }

    // 2) Move backward from the end while each token is normal comment or whitespace
    for t in tokens.iter().rev() {
        if t.text_range().start() < start_offset || t.text_range().end() > end_offset {
            continue;
        }
        if is_whitespace_or_normal_comment(t) {
            let next_end = t.text_range().start();
            if next_end > start_offset {
                end_offset = next_end;
            }
        } else {
            break;
        }
    }

    if start_offset >= end_offset {
        // Everything was normal comment + whitespace
        return TextRange::new(raw.start(), raw.start());
    }
    TextRange::new(start_offset, end_offset)
}

/// True if `t` is whitespace or a normal (non-doc) comment:
fn is_whitespace_or_normal_comment(t: &SyntaxToken) -> bool {
    match t.kind() {
        SyntaxKind::WHITESPACE => true,
        SyntaxKind::COMMENT => {
            // If the text starts with a doc style, keep it in the item
            let txt = t.text();
            // doc line comments begin with `///` or `//!`
            // doc block comments begin with `/**` or `/*!`
            // otherwise it’s normal
            if txt.starts_with("///")
                || txt.starts_with("//!")
                || txt.starts_with("/**")
                || txt.starts_with("/*!")
            {
                false // doc => not normal
            } else {
                true  // normal
            }
        }
        _ => false,
    }
}
