// ---------------- [ File: src/language_model_batch_workflow.rs ]
crate::ix!();

/// This function has been updated so that the generated code correctly:
/// - Implements `ComputeLanguageModelRequests` without referencing non-existent fields like `self.model` or `self.system_message`.
/// - Uses the `model` parameter (passed to `compute_language_model_requests`) rather than ignoring it.
/// - Passes a locally constructed `final_msg` (enhanced system message) instead of referencing `self.system_message`.
/// - Avoids `#[async_trait]` on `ComputeLanguageModelRequests` (since that trait has no async methods).
pub fn generate_impl_language_model_batch_workflow(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_language_model_batch_workflow: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { TokenExpanderError },
    };

    // If the user added `#[batch_json_output_format(SomeType)]`, we set `user_wants_json = true`.
    // Then our macro advises the AI to return well-formed JSON.
    let user_wants_json = parsed.json_output_format_type().is_some();

    let appended_instruction = if user_wants_json {
        let json_output_format_type = parsed.json_output_format_type().as_ref().unwrap();
        quote!{ RigorousJsonCommandBuilderStage::get_all::<#json_output_format_type>() }
    } else {
        quote!{ }
    };

    // The content type used for `execute_language_model_batch_workflow`.
    let content_type_expr = if user_wants_json {
        quote! { ExpectedContentType::Json }
    } else {
        quote! { ExpectedContentType::PlainText }
    };

    // The field the user labeled `#[model_type]` (e.g. `lm_type`).
    let model_type_fld = parsed.model_type_field().as_ref().unwrap();

    // Generate `ComputeLanguageModelRequests` impl block.
    //
    // Note that `ComputeLanguageModelRequests::compute_language_model_requests` now:
    //  1) uses the function's `model: &LanguageModelType` parameter (instead of ignoring it).
    //  2) calls `<Self as ComputeSystemMessage>::system_message()` to get the base system prompt.
    //  3) appends JSON instructions if `user_wants_json` is set.
    //  4) calls `<Self as ComputeLanguageModelCoreQuery>::compute_language_model_core_query(...)`
    //     for each seed item to build the queries.
    //  5) returns `LanguageModelBatchAPIRequest::requests_from_query_strings(&final_msg, model, &queries)`.
    let compute_requests_impl = quote! {
        impl #impl_generics ComputeLanguageModelRequests for #struct_ident #ty_generics #where_clause {
            type Seed = <#struct_ident #ty_generics as ComputeLanguageModelCoreQuery>::Seed;

            fn compute_language_model_requests(
                &self,
                model: &LanguageModelType,
                inputs: &[Self::Seed]
            ) -> Vec<LanguageModelBatchAPIRequest> {
                trace!("Generating language model requests with computed system message and user-wants-json={}.", #user_wants_json);

                let base_msg = <Self as ComputeSystemMessage>::system_message();
                debug!("Base system message is {} chars long.", base_msg.len());

                let final_msg = if #user_wants_json {
                    format!("{}\n\n{}", base_msg, #appended_instruction)
                } else {
                    base_msg
                };
                debug!("Final system message length after JSON note: {}", final_msg.len());

                let mut core_queries = Vec::with_capacity(inputs.len());
                for seed_item in inputs {
                    let req = <Self as ComputeLanguageModelCoreQuery>::compute_language_model_core_query(self, seed_item);
                    core_queries.push(req);
                }
                info!("Built {} core query item(s) from the input seeds.", core_queries.len());

                // Create batch requests from the final system message + each query
                LanguageModelBatchAPIRequest::requests_from_query_strings(
                    &final_msg,
                    model.clone(),
                    &core_queries
                )
            }
        }
    };

    // Generate `LanguageModelBatchWorkflow<Error>` impl block.
    //
    // This block’s `plant_seed_and_wait` method calls `execute_language_model_batch_workflow`
    // with the struct’s own `#[model_type]` field (e.g. `self.lm_type`).
    let workflow_impl = quote! {
        #[async_trait]
        impl #impl_generics LanguageModelBatchWorkflow<#error_type> for #struct_ident #ty_generics #where_clause {

            async fn plant_seed_and_wait(
                &mut self,
                input_tokens: &[<Self as ComputeLanguageModelRequests>::Seed]
            ) -> Result<(), #error_type> {
                debug!("plant_seed_and_wait => auto-generated logic invoked.");
                self.execute_language_model_batch_workflow(
                    self.#model_type_fld.clone(),
                    #content_type_expr.clone(),
                    input_tokens
                ).await
            }
        }
    };

    let expanded = quote! {
        #compute_requests_impl
        #workflow_impl
    };

    trace!("Finished generate_impl_language_model_batch_workflow.");
    expanded
}

#[cfg(test)]
mod test_generate_impl_language_model_batch_workflow {
    use super::*;

    #[traced_test]
    fn generates_impl_avoiding_partial_borrow() {
        info!("Starting generates_impl_avoiding_partial_borrow test.");

        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .batch_client_field(Some(parse_quote! { my_client }))
            .batch_workspace_field(Some(parse_quote! { my_workspace }))
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyErr }))
            .build()
            .unwrap();

        let tokens = generate_impl_language_model_batch_workflow(&parsed);
        let code = tokens.to_string();
        info!("Generated code:\n{}", code);

        // And that we call .await at the end of plant_seed_and_wait
        assert!(code.contains(". await"),
            "We must .await the call to execute_language_model_batch_workflow.");
    }
}
