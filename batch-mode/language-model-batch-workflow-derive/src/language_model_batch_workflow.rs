// ---------------- [ File: src/language_model_batch_workflow.rs ]
crate::ix!();

pub fn generate_impl_language_model_batch_workflow(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_language_model_batch_workflow: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { TokenExpanderError }, // fallback
    };

    let content_type_expr = if let Some(_t) = &parsed.json_output_format_type() {
        quote! { ::batch_mode_batch_workspace_interface::ExpectedContentType::Json }
    } else {
        quote! { ::batch_mode_batch_workspace_interface::ExpectedContentType::PlainText }
    };

    let batch_client       = parsed.batch_client_field().as_ref().unwrap();
    let batch_workspace    = parsed.batch_workspace_field().as_ref().unwrap();
    let system_message_fld = parsed.system_message_field().as_ref().unwrap();
    let model_type_fld     = parsed.model_type_field().as_ref().unwrap();

    // In your macro subroutine:
    let user_wants_json = parsed.json_output_format_type().is_some();

    let compute_requests_impl = quote! {
        #[::async_trait::async_trait]
        impl #impl_generics ComputeLanguageModelRequests for #struct_ident #ty_generics #where_clause {
            type Seed = <#struct_ident #ty_generics as ComputeLanguageModelCoreQuery>::Seed;

            fn compute_language_model_requests(
                &self,
                _unused_model_param: &LanguageModelType,
                inputs: &[Self::Seed]
            ) -> Vec<LanguageModelBatchAPIRequest> {

                tracing::trace!("Auto compute_language_model_requests. Using #[model_type] from the struct instead of the method param.");

                // 1) Grab the system message from the trait method:
                let base_msg = <Self as ComputeSystemMessage>::system_message();

                // If we have a json_output_format, append a clarifying note:
                let final_msg = #user_wants_json {
                    let appended_instruction = format!(
                        "{{}}\\n\\nIMPORTANT: Return valid JSON matching the desired schema. Do not include extraneous text."
                    );
                    format!("{}\\n\\n{}", base_msg, appended_instruction)
                } else {
                    base_msg
                };

                // 2) For each seed, build a request:
                let mut core_queries = Vec::with_capacity(inputs.len());
                for seed_item in inputs {
                    let mut req: String = <Self as ComputeLanguageModelCoreQuery>::compute_language_model_core_query(self, seed_item);
                    core_queries.push(req);
                }
                LanguageModelBatchAPIRequest::requests_from_query_strings(&self.system_message,self.model,&core_queries)
            }
        }
    };

    let workflow_impl = quote! {
        #[::async_trait::async_trait]
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

    quote! {
        #compute_requests_impl
        #workflow_impl
    }
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
            .system_message_field(Some(parse_quote! { sm }))
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
