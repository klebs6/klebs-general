// ---------------- [ File: src/language_model_batch_workflow.rs ]
crate::ix!();

/// Generate the `impl LanguageModelBatchWorkflow<Error>` code.  
/// If user sets `#[batch_error_type(...)]`, we use that. Otherwise we default to `TokenExpanderError`.
pub fn generate_impl_language_model_batch_workflow(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_language_model_batch_workflow: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote!{ #t },
        None => quote!{ TokenExpanderError },
    };

    let model_type_field = match &parsed.model_type_field() {
        Some(t) => quote!{ #t },
        None => panic!("model_type_field is mandatory"),
    };

    let expected_content_type_field = match &parsed.expected_content_type_field() {
        Some(t) => quote!{ #t },
        None => panic!("expected_content_type_field is mandatory"),
    };

    quote! {
        #[::async_trait::async_trait]
        impl #impl_generics LanguageModelBatchWorkflow<#error_type> for #struct_ident #ty_generics #where_clause {

            async fn plant_seed_and_wait(
                &mut self,
                input_tokens:          &[<Self as ComputeLanguageModelRequests>::Seed]
            ) -> Result<(),#error_type> {
                self.execute_language_model_batch_workflow(
                    &self.#model_type_field,
                    &self.#expected_content_type_field,
                    input_tokens
                )
            }
        }
    }
}

#[cfg(test)]
mod test_generate_impl_language_model_batch_workflow {
    use super::*;

    #[traced_test]
    fn generates_empty_impl_for_lmbw() {
        info!("Starting generates_empty_impl_for_lmbw test.");

        // If we are testing that it only generates an empty impl block,
        // we still must supply all "required" fields so the builder
        // won't fail. "Empty" in the sense that the trait is implemented
        // but has no methods. We'll skip the optional pbo/pbe fields.

        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .where_clause(None)
            .batch_client_field(Some(parse_quote! { my_client }))
            .batch_workspace_field(Some(parse_quote! { my_workspace }))
            .expected_content_type_field(Some(parse_quote! { ect }))
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyErr }))
            .build()
            .unwrap();

        let tokens = generate_impl_language_model_batch_workflow(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        assert!(
            code.contains("impl LanguageModelBatchWorkflow < MyErr > for Dummy"),
            "Should create an empty LanguageModelBatchWorkflow impl with the provided custom error type."
        );
    }
}
