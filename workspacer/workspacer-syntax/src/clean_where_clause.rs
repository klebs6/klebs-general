crate::ix!();

pub fn full_clean_where_clause(x: &Option<ast::WhereClause>) -> String {

    // 6) Where clause
    let raw_where    = x
        .clone()
        .map(|wc| wc.syntax().text().to_string())
        .unwrap_or_default();

    // NOTE: yeah, we fucking *blast* those where clause comments cause fuck em!
    // FIX: no! bad! we comment your blasting!
    // let raw_where  = remove_comments_in_where_clause(&raw_where);

    clean_where_clause(&raw_where)
}

/// **New helper** that blasts all `//...` line comments and `/*...*/` block comments
/// from the given `where`-clause text.
pub fn remove_comments_in_where_clause(where_text: &str) -> String {
    // Very naive approach: remove block comments, remove `//` lines, done.

    // 1) Remove block comments by replacing `/* ... */` with "" repeatedly:
    let mut result = where_text.to_string();
    while let Some(start_idx) = result.find("/*") {
        if let Some(end_idx) = result[start_idx..].find("*/") {
            let actual_end = start_idx + end_idx + 2; // +2 to include "*/"
            result.replace_range(start_idx..actual_end, "");
        } else {
            // No closing */ => remove from start_idx to end
            result.replace_range(start_idx.., "");
            break;
        }
    }

    // 2) Remove all `//...` line comments. We'll do it line by line:
    let mut out_lines = Vec::new();
    for line in result.lines() {
        if let Some(pos) = line.find("//") {
            // Keep everything up to the `//`
            out_lines.push(line[..pos].to_string());
        } else {
            out_lines.push(line.to_string());
        }
    }

    // 3) Rejoin them with newlines, so flatten_whitespace can do its job later.
    out_lines.join("\n")
}

/// Removes trailing commas from a `where` clause flattened to one line.
/// If it's just "where", remove it entirely.
pub fn clean_where_clause(text: &str) -> String {
    if !text.starts_with("where") {
        return text.to_string();
    }
    let trimmed = text.trim_end_matches(',').trim();
    if trimmed == "where" {
        "".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Splits on all whitespace and rejoins with one space, trimming.
pub fn flatten_whitespace(text: &str) -> String {
    let tokens: Vec<_> = text.split_whitespace().collect();
    tokens.join(" ")
}
