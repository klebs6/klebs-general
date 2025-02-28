crate::ix!();

/// Builds the final snippet for the "has_imports_line=true" scenario:
/// (a) old_top_macros first (with their leading comments),
/// (b) then non-macro lines,
/// (c) then new_top_macros.
pub fn build_top_block_for_imports_line(
    old_top_macros: &[TopBlockMacro],
    non_macro_lines: &[String],
    new_top_macros: &[TopBlockMacro],
) -> String {
    trace!("Entering build_top_block_for_imports_line");
    let mut buffer = String::new();

    // 1) old macros
    for om in old_top_macros {
        maybe_push_newline(&mut buffer);
        push_macro_with_comments(&mut buffer, om);
    }

    // 2) snippet lines
    for (i, line) in non_macro_lines.iter().enumerate() {
        maybe_push_newline(&mut buffer);
        buffer.push_str(line);
        // if not last line => push another newline
        if i < non_macro_lines.len() - 1 {
            buffer.push('\n');
        }
    }

    // 3) new macros
    for nm in new_top_macros {
        maybe_push_newline(&mut buffer);
        push_macro_with_comments(&mut buffer, nm);
    }

    debug!("build_top_block_for_imports_line => length={}", buffer.len());
    trace!("Exiting build_top_block_for_imports_line");
    buffer
}
