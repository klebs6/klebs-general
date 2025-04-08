crate::ix!();

/// Joins a list of strings with a blank line separating them.
pub fn join_with_blank_line(list: Vec<String>) -> String {
    if list.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (i, line) in list.iter().enumerate() {
        out.push_str(line);
        out.push('\n');
        if i + 1 < list.len() {
            out.push('\n');
        }
    }
    out
}
