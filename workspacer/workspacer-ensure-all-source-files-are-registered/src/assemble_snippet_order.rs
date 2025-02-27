crate::ix!();

// Helper to produce: [macrosA] + [snippet lines] + [macrosB], all joined with newlines
pub fn assemble_snippet_order(
    macros_a: &[TopBlockMacro],
    snippet_lines: &[String],
    macros_b: &[TopBlockMacro],
) -> String {
    // We'll just push them in sequence, each separated by newlines as needed.
    let mut buf = String::new();

    // macros_a
    for mac in macros_a {
        if !buf.is_empty() && !buf.ends_with('\n') {
            buf.push('\n');
        }
        // If macro has leading comments:
        if !mac.leading_comments().is_empty() {
            buf.push_str(mac.leading_comments());
            if !buf.ends_with('\n') {
                buf.push('\n');
            }
        }
        // Then macro line
        buf.push_str(&format!("x!{{{}}}", mac.stem()));
    }

    // snippet lines
    for (i, line) in snippet_lines.iter().enumerate() {
        if !buf.is_empty() && !buf.ends_with('\n') {
            buf.push('\n');
        }
        buf.push_str(line);
        if i < snippet_lines.len() - 1 {
            buf.push('\n');
        }
    }

    // macros_b
    for mac in macros_b {
        if !buf.is_empty() && !buf.ends_with('\n') {
            buf.push('\n');
        }
        // Leading comments
        if !mac.leading_comments().is_empty() {
            buf.push_str(mac.leading_comments());
            if !buf.ends_with('\n') {
                buf.push('\n');
            }
        }
        buf.push_str(&format!("x!{{{}}}", mac.stem()));
    }

    // Trim trailing newlines
    while buf.ends_with('\n') {
        buf.pop();
    }

    buf
}
