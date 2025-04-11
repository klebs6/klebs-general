// ---------------- [ File: workspacer-syntax/src/rehydrate_nodes.rs ]
crate::ix!();

impl RehydrateFromSignature for ast::Fn {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::Struct {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::Enum {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::Trait {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::TypeAlias {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::MacroRules {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

impl RehydrateFromSignature for ast::MacroCall {
    #[tracing::instrument(level = "trace", skip(signature_source))]
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
        parse_exact_one_top_level_item_of_type(signature_source)
    }
}

/// Attempts to parse `signature_source` as **exactly one** top-level item of type `T`.
///
/// **Checks**:
/// 1. `parse.errors()` is empty (the snippet parses without syntax errors).
/// 2. Exactly one top-level `ast::Item` is present.
/// 3. That single item can be cast to `T` (e.g. `ast::Fn`, `ast::Struct`, etc.).
/// 4. Any trailing text after the item is *only* whitespace or line comments (`//...`)
///    or block comments. If we detect real tokens (e.g. code) after the item, we fail.
///
/// If all checks pass, we return `Some(T)`. Otherwise, `None`.
#[tracing::instrument(level = "trace", skip(signature_source))]
fn parse_exact_one_top_level_item_of_type<T: AstNode>(signature_source: &str) -> Option<T> {
    trace!("Parsing snippet:\n{}", signature_source);

    let parse = SourceFile::parse(signature_source, Edition::Edition2021);
    let sf = parse.tree();
    let parse_errors = parse.errors();

    if !parse_errors.is_empty() {
        // You could log or store these errors if you wish:
        warn!("Parse errors found in snippet => not rehydrating. Errors: {:?}", parse_errors);
        return None;
    }

    // Gather top-level items
    let items: Vec<ast::Item> = sf.items().collect();
    if items.len() != 1 {
        debug!(
            "Expected exactly 1 top-level item, found {} => not rehydrating.",
            items.len()
        );
        return None;
    }

    // Cast that one item to T
    let only_item = &items[0];
    if let Some(desired_node) = T::cast(only_item.syntax().clone()) {
        // Now ensure that there's no extraneous text beyond the item except
        // for whitespace/comments. We'll compare the item’s text range to the full file range.
        let item_range: TextRange = only_item.syntax().text_range();
        let file_range: TextRange = sf.syntax().text_range();

        // If the item ends before the file ends, let's see what's after it
        if item_range.end() < file_range.end() {
            let file_text = sf.syntax().text();
            let trailing_slice = file_text.slice(item_range.end()..file_range.end());
            let trailing_str = trailing_slice.to_string();

            // If the trailing text (minus whitespace and comment-like strings) is not empty,
            // we fail. We'll do a naive check: remove all types of whitespace plus
            // `//` lines or `/*...*/`.
            // We can do a more thorough check if needed; for now we keep it simple.
            let cleaned = remove_comments_and_whitespace(&trailing_str);
            if !cleaned.is_empty() {
                debug!("Found extra tokens after the item: {:?}", cleaned);
                return None;
            }
        }

        trace!("Snippet rehydrated successfully => returning T");
        Some(desired_node)
    } else {
        debug!("Top-level item is not the correct type => cast failed.");
        None
    }
}

/// Remove all forms of whitespace and line/block comments from a string, returning whatever remains.
/// A simplistic approach: for each line, we trim whitespace. If it starts with `//`, we remove it entirely.
/// For block comments `/*...*/`, we do a naive `.replace(...)` approach. Real code might do a small parse.
fn remove_comments_and_whitespace(input: &str) -> String {
    // Remove block comments
    let mut no_block = input.replace("/*", "\u{1}").replace("*/", "\u{2}");
    // naive: everything between \u{1} and \u{2} is removed
    while let Some(start) = no_block.find('\u{1}') {
        if let Some(end) = no_block[start..].find('\u{2}') {
            let absolute_end = start + end + 1; // +1 to skip the \u{2}
            no_block.replace_range(start..absolute_end, "");
        } else {
            // unclosed block comment => treat as parse error => we won't handle partial
            // but let's just remove from start to end of string
            no_block.replace_range(start..no_block.len(), "");
        }
    }
    // Remove the placeholders
    no_block = no_block.replace('\u{1}', "").replace('\u{2}', "");

    // Now handle lines
    let mut cleaned = String::new();
    for line in no_block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            // skip entire line
            continue;
        }
        // remove leading + trailing whitespace
        let l = line.trim();
        if !l.is_empty() {
            cleaned.push_str(l);
            cleaned.push('\n');
        }
    }
    cleaned.trim().to_string()
}
