// ---------------- [ File: src/language_model_batch_workflow.rs ]
crate::ix!();

/// Generate the `impl LanguageModelBatchWorkflow<Error>` code.
/// We simply have `plant_seed_and_wait` call `execute_language_model_batch_workflow`
/// using `&mut self`, and inside the latter, we read our `model_type_field`
/// and `expected_content_type_field` from `self`.
///
/// This avoids the partial-borrow error by ensuring all references to `self`
/// come from a single `&mut self` borrow.
pub fn generate_impl_language_model_batch_workflow(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_language_model_batch_workflow: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { TokenExpanderError },
    };

    // We’ll just remember which fields had #[model_type] and #[expected_content_type],
    // but we won't pass them as separate function parameters. Instead, we read them
    // inside the function, to avoid partial-borrow conflicts.
    let model_type_field = match &parsed.model_type_field() {
        Some(id) => quote! { #id },
        None => panic!("model_type_field is mandatory"),
    };
    let expected_content_type_field = match &parsed.expected_content_type_field() {
        Some(id) => quote! { #id },
        None => panic!("expected_content_type_field is mandatory"),
    };

    quote! {
        #[::async_trait::async_trait]
        impl #impl_generics LanguageModelBatchWorkflow<#error_type> for #struct_ident #ty_generics #where_clause {

            async fn plant_seed_and_wait(
                &mut self,
                input_tokens: &[<Self as ComputeLanguageModelRequests>::Seed]
            ) -> Result<(), #error_type> {
                tracing::debug!("plant_seed_and_wait => calling our main workflow method (mutable).");
                self.execute_language_model_batch_workflow(
                    self.#model_type_field.clone(),
                    self.#expected_content_type_field.clone(),
                    input_tokens
                ).await
            }
        }
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
            .expected_content_type_field(Some(parse_quote! { ect }))
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
