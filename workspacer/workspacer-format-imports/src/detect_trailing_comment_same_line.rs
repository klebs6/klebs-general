crate::ix!();

/// Detects a same-line trailing `// comment` starting just after offset `pos`
/// (e.g. after the semicolon). If found, returns `(comment_text, total_length)`
/// so we can store that comment text and expand the removal range.
/// If none found, returns None.
pub fn detect_trailing_comment_same_line(old_text: &str, pos: usize) -> Option<(String, usize)> {
    info!("detect_trailing_comment_same_line => start; pos={}", pos);
    if pos >= old_text.len() {
        debug!("Position >= old_text.len() => no trailing comment");
        return None;
    }

    // We'll scan up to the next newline or end-of-file
    let line_end = match old_text[pos..].find('\n') {
        Some(rel) => pos + rel,
        None => old_text.len(),
    };

    // The substring from pos..line_end is the remainder on that line
    let candidate = &old_text[pos..line_end];
    debug!("Same-line candidate is {:?}", candidate);

    // We'll skip any leading spaces/tabs after the semicolon
    let trimmed_start = candidate.find(|c: char| !c.is_whitespace()).unwrap_or(candidate.len());
    let check_pos = pos + trimmed_start;
    if check_pos + 2 <= line_end && &old_text[check_pos..check_pos + 2] == "//" {
        // The comment is from check_pos..line_end
        let comment = old_text[check_pos..line_end].to_string();
        // The total length covers from pos..line_end
        let total_len = line_end - pos;
        info!("detect_trailing_comment_same_line => found comment => {:?}", comment);
        Some((comment, total_len))
    } else {
        debug!("No trailing '//' found => returning None");
        None
    }
}
