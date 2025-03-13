// ---------------- [ File: src/language_model_batch_workflow.rs ]
crate::ix!();

/// This function has been updated so that the generated code correctly handles
/// the optional appended instructions without triggering format-string errors.
pub fn generate_impl_language_model_batch_workflow(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_language_model_batch_workflow: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { TokenExpanderError },
    };

    let user_wants_json = parsed.json_output_format_type().is_some();

    // We'll embed the final message-building logic so it never produces
    // a mismatched argument count.
    let msg_construction_expr = if user_wants_json {
        let json_ty = parsed.json_output_format_type().as_ref().unwrap();
        quote! {
            {
                let base_msg = <Self as ComputeSystemMessage>::system_message();
                let appended = RigorousJsonCommandBuilderStage::get_all::<#json_ty>();
                format!("{}\n\n{}", base_msg, appended)
            }
        }
    } else {
        quote! {
            {
                <Self as ComputeSystemMessage>::system_message()
            }
        }
    };

    let model_type_fld = parsed.model_type_field().as_ref().unwrap();

    let compute_requests_impl = quote! {
        impl #impl_generics ComputeLanguageModelRequests for #struct_ident #ty_generics #where_clause {
            type Seed = <#struct_ident #ty_generics as ComputeLanguageModelCoreQuery>::Seed;

            fn compute_language_model_requests(
                &self,
                model: &LanguageModelType,
                inputs: &[Self::Seed]
            ) -> Vec<LanguageModelBatchAPIRequest> {
                tracing::trace!("Generating language model requests; user-wants-json={}", #user_wants_json);

                // Construct the final system prompt with or without JSON instructions
                let final_msg = #msg_construction_expr;
                tracing::debug!("Final system message length is {}", final_msg.len());

                let mut core_queries = Vec::with_capacity(inputs.len());
                for seed_item in inputs {
                    let req = <Self as ComputeLanguageModelCoreQuery>::compute_language_model_core_query(self, seed_item);
                    core_queries.push(req);
                }
                tracing::info!("Built {} core query item(s) from the input seeds.", core_queries.len());

                LanguageModelBatchAPIRequest::requests_from_query_strings(
                    &final_msg,
                    model.clone(),
                    &core_queries
                )
            }
        }
    };

    let content_type_expr = if user_wants_json {
        quote! { ExpectedContentType::Json }
    } else {
        quote! { ExpectedContentType::PlainText }
    };

    let workflow_impl = quote! {
        #[async_trait]
        impl #impl_generics LanguageModelBatchWorkflow<#error_type> for #struct_ident #ty_generics #where_clause {

            async fn plant_seed_and_wait(
                &mut self,
                input_tokens: &[<Self as ComputeLanguageModelRequests>::Seed]
            ) -> Result<(), #error_type> {
                tracing::debug!("plant_seed_and_wait => auto-generated logic invoked.");
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

    tracing::trace!("Finished generate_impl_language_model_batch_workflow.");
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
