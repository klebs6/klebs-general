#![allow(unused_imports)]
//#![allow(unused_mut)]
//#![allow(unused_assignments)]
//#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use] mod imports; use imports::*;

x!{build_effective_config_from_cli}
x!{cli}
x!{effective_config}
x!{errors}
x!{extract_items_from_ast}
x!{extract_signature}
x!{filter_criteria}
x!{function_info}
x!{gather_all_code_from_crates_main}
x!{global_config}
x!{item_info}
x!{process_crate_directory}
x!{process_file}
x!{reconstruct}
x!{remove_unwanted_lines}
x!{extract_attributes}
x!{filter_doc_comments}
x!{parse_function_item}
x!{parse_struct_item}
x!{parse_enum_item}
x!{parse_source_code}
x!{parse_type_alias_item}
x!{parse_impl_block_item}
x!{filter_attributes}
