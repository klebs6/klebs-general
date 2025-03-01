// ---------------- [ File: src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{assemble_final_top_block_snippet}
x!{build_top_block_for_imports_line}
x!{build_top_block_for_no_imports_line}
x!{collect_existing_macro_stems}
x!{collect_existing_x_macros}
x!{crate_ensure_all_source_files_are_registered}
x!{create_top_block_text}
x!{determine_top_block_insertion_offset}
x!{ensure}
x!{existing_macros_to_top_block_macros}
x!{existing_x_macro}
x!{extract_non_macro_lines}
x!{extract_stem}
x!{file_has_imports_line}
x!{find_earliest_non_macro_item_offset}
x!{find_last_import_end_before_offset}
x!{find_top_block_insertion_offset}
x!{gather_deduplicated_macro_stems}
x!{gather_leading_comments}
x!{gather_old_top_block_macros}
x!{is_imports_line}
x!{is_x_macro}
x!{make_top_block_macro_lines}
x!{parse_new_macros_with_comments}
x!{parse_new_top_block_snippet}
x!{push_macro_with_comments}
x!{rebuild_librs_with_new_top_block}
x!{remove_trailing_newlines}
x!{snap_offset_to_newline}
x!{splice_top_block_into_source}
x!{workspace_ensure_all_source_files_are_registered}
x!{filter_new_macros_for_duplicates}
x!{maybe_push_newline}
x!{maybe_ensure_newline}
