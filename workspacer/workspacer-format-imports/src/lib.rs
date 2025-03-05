// ---------------- [ File: workspacer-format-imports/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{build_new_use_lines}
x!{collect_comment_token}
x!{combine_new_uses}
x!{dispatch_token_by_kind}
x!{dissect_use_statement}
x!{errors}
x!{fallback_scan_node_text}
x!{gather_comments_state}
x!{gather_leading_comment_lines}
x!{gather_leading_token_comments}
x!{gather_sibling_comments_above}
x!{gather_token_comments_above}
x!{gather_use_items}
x!{group_and_sort_uses}
x!{parse_and_validate_syntax}
x!{peek_next_non_whitespace}
x!{process_other_token}
x!{process_upward_node}
x!{process_whitespace_token}
x!{remove_old_use_statements}
x!{scan_preceding_tokens_for_comments}
x!{skip_node_plus_trailing_whitespace}
x!{skip_upward_node_with_whitespace}
x!{sort_and_format_imports_for_crate}
x!{sort_and_format_imports_for_workspace}
x!{sort_and_format_imports}
x!{split_path_into_prefix_and_final}
x!{use_item_info}
x!{detect_trailing_comment_same_line}
