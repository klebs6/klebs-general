// ---------------- [ File: src/finish_processing_uncompleted_batches.rs ]
crate::ix!();

/// Generate the `impl FinishProcessingUncompletedBatches` code.
pub fn generate_impl_finish_processing_uncompleted_batches(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_finish_processing_uncompleted_batches: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote!{ #t },
        None    => quote!{ TokenExpanderError },
    };

    //todo: this one needs to be required, and if we don't have it, we should provide the user with
    //a well written compiler error
    let workspace_expr = if let Some(w) = &parsed.batch_workspace_field() {
        quote!{ self.#w.clone() }
    } else {
        quote!{ self.workspace() }
    };

    //todo: this one also needs to be required, and if we don't have it, we should provide the user with
    //a well written compiler error
    let client_expr = if let Some(c) = &parsed.batch_client_field() {
        quote!{ self.#c.clone() }
    } else {
        quote!{ self.client() }
    };
    let output_fn_expr = if let Some(out_fn) = &parsed.process_batch_output_fn_field() {
        quote!{ &self.#out_fn }
    } else {
        quote!{ &PROCESS_OUTPUT_FILE_BRIDGE }
    };
    let error_fn_expr = if let Some(err_fn) = &parsed.process_batch_error_fn_field() {
        quote!{ &self.#err_fn }
    } else {
        quote!{ &PROCESS_ERROR_FILE_BRIDGE }
    };

    quote! {
        #[::async_trait::async_trait]
        impl #impl_generics FinishProcessingUncompletedBatches for #struct_ident #ty_generics #where_clause {
            type Error = #error_type;

            async fn finish_processing_uncompleted_batches(&self, expected_content_type: &batch_mode_batch_workspace_interface::ExpectedContentType)
                -> Result<(), Self::Error>
            {
                tracing::info!("Finishing uncompleted batches if any remain.");
                let workspace = #workspace_expr;
                let language_model_client = #client_expr;

                let mut batch_triples = workspace.clone().gather_all_batch_triples().await?;
                tracing::info!("Reconciling unprocessed batch files in the work directory");

                for triple in &mut batch_triples {
                    triple.reconcile_unprocessed(
                        &*language_model_client,
                        expected_content_type,
                        #output_fn_expr,
                        #error_fn_expr
                    ).await?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test_generate_impl_finish_processing_uncompleted_batches {
    use super::*;

    #[traced_test]
    fn generates_impl_with_all_fields() {
        info!("Starting generates_impl_with_all_fields test.");

        // Provide *all* required fields plus pbo/pbe. That ensures 
        // the subroutine sees them and references them.

        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .batch_client_field(Some(parse_quote! { some_client }))
            .batch_workspace_field(Some(parse_quote! { some_workspace }))
            .process_batch_output_fn_field(Some(parse_quote! { pbo }))
            .process_batch_error_fn_field(Some(parse_quote! { pbe }))
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyErr }))
            .build()
            .unwrap();

        let tokens = generate_impl_finish_processing_uncompleted_batches(&parsed);
        let code   = tokens.to_string();
        info!("Generated code: {}", code);

        assert!(code.contains("impl FinishProcessingUncompletedBatches for Dummy"),
                "Should impl trait for struct 'Dummy'.");
        assert!(code.contains("self . some_workspace . clone ()"),
                "Should reference the workspace field we found.");
        assert!(code.contains("self . some_client . clone ()"),
                "Should reference the client field we found.");
        assert!(code.contains("self . pbo"),
                "Should reference the custom process batch output fn we found.");
        assert!(code.contains("self . pbe"),
                "Should reference the custom process batch error fn we found.");
    }

    #[traced_test]
    fn generates_impl_without_custom_output_error_fns() {
        info!("Starting generates_impl_without_custom_output_error_fns test.");

        // Provide required fields but skip the optional pbo/pbe. 
        // That means subroutine falls back to PROCESS_OUTPUT_FILE_BRIDGE, etc.

        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .batch_client_field(Some(parse_quote! { some_client }))
            .batch_workspace_field(Some(parse_quote! { some_workspace }))
            // skip process_batch_output_fn_field => None
            // skip process_batch_error_fn_field => None
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyErr }))
            .build()
            .unwrap();

        let tokens = generate_impl_finish_processing_uncompleted_batches(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        assert!(code.contains("& PROCESS_OUTPUT_FILE_BRIDGE"),
                "Should fallback to PROCESS_OUTPUT_FILE_BRIDGE.");
        assert!(code.contains("& PROCESS_ERROR_FILE_BRIDGE"),
                "Should fallback to PROCESS_ERROR_FILE_BRIDGE.");
    }
}
