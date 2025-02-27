// ---------------- [ File: src/create_top_block_text.rs ]
crate::ix!();

pub fn create_top_block_text(tops: &[TopBlockMacro]) -> String {
    // You might have some initial comment lines or text you want at top:
    let mut buffer = String::new();

    for top in tops {
        // Insert the user’s leading comments (which might include doc lines or `// ...`)
        if !top.leading_comments().is_empty() {
            // Ensure we separate them from prior macro with a newline if needed
            if !buffer.ends_with('\n') {
                buffer.push('\n');
            }
            buffer.push_str(&top.leading_comments());
        }
        // Then add the macro line
        buffer.push_str(&format!("x!{{{}}}\n", top.stem()));
    }

    // Trim trailing newlines
    while buffer.ends_with('\n') {
        buffer.pop();
    }

    buffer
}
