// ---------------- [ File: batch-mode-process-response/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{process_batch_output_and_errors}
x!{process_error_data}
x!{process_output_data}
x!{process_output_file}
x!{process_error_file}
x!{handle_finish_reason_length}
x!{handle_successful_response}
x!{handle_failed_json_repair}
x!{save_failed_entries}
