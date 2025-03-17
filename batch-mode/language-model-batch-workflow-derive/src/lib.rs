// ---------------- [ File: src/lib.rs ]
//! This module contains the procedural macro for deriving `LanguageModelBatchWorkflow`.
//! We break down the macro into discrete subroutines for clearer testing and maintainability.
//! Each subroutine has a corresponding test module that validates its behavior at an interface level.
//!
//! We must place this file in our `proc-macro` crate (e.g. `src/lib.rs`), adjusting as appropriate.
//! Remember to add `syn = "2.0"`, `quote = "1.0"`, `proc-macro2 = "1.0"`, and `async-trait = "0.1"`
//! to your `Cargo.toml`. This macro also uses the `tracing` crate for robust logging.

#[macro_use] mod imports; use imports::*;

xp!{combine_impls_into_final_macro}
xp!{finish_processing_uncompleted_batches}
xp!{get_batch_workspace}
xp!{get_language_model_client}
xp!{language_model_batch_workflow}
xp!{process_batch_requests}
xp!{send_sync}
xp!{lmbw_parsed_input}
xp!{parse_derive_input_for_lmbw}
xp!{gather_results}

#[proc_macro_derive(
    LanguageModelBatchWorkflow,
    attributes(
        batch_json_output_format,
        batch_client,
        batch_workspace,
        custom_process_batch_output_fn,
        custom_process_batch_error_fn,
        expected_content_type,
        model_type,
        batch_error_type
    )
)]
pub fn language_model_batch_workflow_derive(input: TokenStream) -> TokenStream {
    tracing::trace!("Entering language_model_batch_workflow_derive proc macro.");

    let ast: DeriveInput = syn::parse_macro_input!(input as DeriveInput);

    let parse_result = match parse_derive_input_for_lmbw(&ast) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };

    // existing sub-impls
    let finish_processing_impl      = generate_impl_finish_processing_uncompleted_batches(&parse_result);
    let process_batch_requests_impl = generate_impl_process_batch_requests(&parse_result);
    let workflow_impl               = generate_impl_language_model_batch_workflow(&parse_result);
    let send_sync_impl              = generate_impl_send_sync(&parse_result);
    let get_workspace_impl          = generate_impl_get_batch_workspace(&parse_result);
    let get_client_impl             = generate_impl_get_language_model_client(&parse_result);

    // new gather results trait impl
    let gather_results_trait_impl   = generate_impl_gather_results_trait(&parse_result);

    let expanded = combine_impls_into_final_macro(vec![
        finish_processing_impl,
        process_batch_requests_impl,
        workflow_impl,
        send_sync_impl,
        get_workspace_impl,
        get_client_impl,
        gather_results_trait_impl,
    ]);

    tracing::trace!("Exiting language_model_batch_workflow_derive proc macro.");
    expanded.into()
}
