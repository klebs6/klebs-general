// ---------------- [ File: batch-mode-batch-client/src/lib.rs ]
#![feature(trait_alias)]

#[macro_use] mod imports; use imports::*;

x!{batch_online_status}
x!{check_and_download_interface}
x!{check_and_download_output_and_error_online}
x!{check_batch_status_online}
x!{create_batch}
x!{download_error_file}
x!{download_output_file}
x!{errors}
x!{get_batch_file_content}
x!{impl_language_model_client_interface_for_arc_dyn}
x!{language_model_client_interface}
x!{mock}
x!{openai_client_handle}
x!{retrieve_batch_by_id}
x!{upload_batch_file}
x!{wait_for_batch_completion}
