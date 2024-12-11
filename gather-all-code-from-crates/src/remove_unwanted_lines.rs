crate::ix!();

/// Removes doc comment lines (`///` or `//!`) if `remove_doc_comments = true`.
/// Keeps all other lines intact, including attributes (`#[...]`), normal `//` comments,
/// indentation, and whitespace. No inline truncation or other modifications are made.
pub fn new_filter_lines(s: String, remove_doc_comments: bool) -> String {
    let mut filtered = Vec::new();
    for line in s.lines() {
        let trimmed = line.trim_start();

        // If requested to remove doc comments, skip lines starting with `///` or `//!`
        if remove_doc_comments && (trimmed.starts_with("///") || trimmed.starts_with("//!")) {
            continue;
        }

        // Keep everything else intact
        filtered.push(line);
    }

    filtered.join("\n")
}

pub fn filter_lines(raw_signature: String, remove_doc_comments: bool) -> String {
    if remove_doc_comments {
        remove_unwanted_lines(raw_signature)
    } else {
        remove_unwanted_lines_keep_docs(raw_signature)
    }
}

/// Removes all doc comments (`///`, `//!`), attribute lines (`#[...]`), and normal `//` comments 
/// when `remove_doc_comments = true`. Also handles inline `//` in code lines by truncation, preserving whitespace.
pub fn remove_unwanted_lines(s: String) -> String {
    let mut filtered = Vec::new();

    for line in s.lines() {
        let trimmed = line.trim_start();

        // Identify line type:
        // 1. Attribute line starts with `#[`
        if trimmed.starts_with("#[") {
            // Remove entire line
            continue;
        }

        // 2. Doc lines `///` or `//!`
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            // remove_doc_comments=true means remove doc lines entirely
            continue;
        }

        // 3. Normal `//` lines (with no code) - remove entirely
        // A normal `//` line is one that starts with `//` but is not doc line.
        // Since doc lines are already handled, if it starts with `//` here, it's a normal comment line.
        if trimmed.starts_with("//") {
            // remove entire normal `//` line
            continue;
        }

        // 4. Inline `//` in code lines:
        // If there's a `//` somewhere after code, truncate the line at `//`.
        if let Some(idx) = line.find("//") {
            // Keep indentation and code before `//`
            let code_part = &line[..idx];
            // If code_part is all whitespace (meaning line was effectively a comment?), skip if needed.
            // But here we have code line with inline comment. Keep code_part even if it's whitespace,
            // since the user wants whitespace preserved.
            let trimmed_code_part = code_part; 
            // Only add if something remains after truncation
            if !trimmed_code_part.is_empty() {
                filtered.push(trimmed_code_part);
            } else {
                // If after truncation nothing remains, it means the line was basically empty or just spaces before `//`.
                // remove_doc_comments=true means we remove this line, as it's effectively a comment line.
                continue;
            }
        } else {
            // No `//` found, keep the line as is
            filtered.push(line);
        }
    }

    filtered.join("\n")
}

/// Removes attribute lines (`#[...]`) and normal `//` comment lines, but keeps doc lines (`///`, `//!`) 
/// when `remove_doc_comments = false`. Also handles inline `//` in code lines by truncation, preserving whitespace.
///
/// Doc lines are kept intact. Inline `//` in code lines still gets truncated.
pub fn remove_unwanted_lines_keep_docs(s: String) -> String {
    let mut filtered = Vec::new();

    for line in s.lines() {
        let trimmed = line.trim_start();

        // Attribute lines:
        if trimmed.starts_with("#[") {
            // Remove entire line
            continue;
        }

        // Normal `//` comment lines (not doc lines):
        // Doc lines start with `///` or `//!`. If line starts with `//` and it's not doc, it's a normal comment line.
        if trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!") {
            // Remove entire normal `//` comment line
            continue;
        }

        // If we get here and the line starts with `///` or `//!`, it's a doc line and we keep it as is.

        // Inline `//` in code lines:
        // If there's `//` somewhere, and it's not at the start, we truncate at `//`.
        // For doc lines (`///`, `//!`), we keep them intact, so only truncate if it's a code line.
        if let Some(idx) = line.find("//") {
            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                // It's a doc line with `//` inside it (rare), keep entire line.
                filtered.push(line);
            } else {
                // Code line with inline `//`, truncate at `//`.
                let code_part = &line[..idx];
                if !code_part.is_empty() {
                    filtered.push(code_part);
                } else {
                    // Nothing left but whitespace, effectively a comment line => skip
                    continue;
                }
            }
        } else {
            // No `//` found, keep line as is
            filtered.push(line);
        }
    }

    filtered.join("\n")
}

