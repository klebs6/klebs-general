#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(trait_alias)]
#[macro_use] mod imports; use imports::*;

x!{repair_json_accidental_single_quote_instead_of_a_double_quote}
x!{repair_json_add_missing_quotes}
x!{repair_json_attempt}
x!{repair_json_close_unexpected_eof_in_array_item}
x!{repair_json_close_unexpected_eof_in_array_tag}
x!{repair_json_close_unexpected_eof}
x!{repair_json_comma_behavior}
x!{repair_json_control_characters}
x!{repair_json_fix_mismatched_quotes}
x!{repair_json_handle_eof_between_lists}
x!{repair_json_mismatched_brackets}
x!{repair_json_missing_commas_in_list}
x!{repair_json_remove_control_characters_in_value}
x!{repair_json_remove_duplicate_quotes}
x!{repair_json_truncated_boolean_behavior}
x!{repair_json}
x!{errors}
x!{sanitize}
