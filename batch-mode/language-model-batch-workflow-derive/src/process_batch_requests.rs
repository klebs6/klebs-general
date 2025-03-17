// ---------------- [ File: src/process_batch_requests.rs ]
crate::ix!();

/// Generate the `impl ProcessBatchRequests` code. We rely on:
///
/// - `batch_workspace_field` => used for `workspace`
/// - `batch_client_field`    => used for `client` calls
/// - `custom_process_batch_output_fn` => might be used if the user
///   wants a specialized success path. We do your snippet's default logic:
///   calling `process_batch_output_and_errors(...)`.
/// - `expected_content_type_field` => used as a parameter
pub fn generate_impl_process_batch_requests(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_process_batch_requests: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote!{ #t },
        None => quote!{ TokenExpanderError },
    };

    let workspace_expr = if let Some(w) = &parsed.batch_workspace_field() {
        quote!{ self.#w.clone() }
    } else {
        quote!{ self.workspace() }
    };
    let client_expr = if let Some(c) = &parsed.batch_client_field() {
        quote!{ self.#c.clone() }
    } else {
        quote!{ self.client() }
    };

    let user_output_ty = match parsed.json_output_format_type() {
        Some(t) => quote! { #t },
        None    => quote! { CamelCaseTokenWithComment },
    };

    quote! {
        #[async_trait]
        impl #impl_generics ProcessBatchRequests for #struct_ident #ty_generics #where_clause {
            type Error = #error_type;

            async fn process_batch_requests(
                &self, 
                batch_requests:        &[LanguageModelBatchAPIRequest], 
                expected_content_type: &ExpectedContentType)
                -> Result<(), Self::Error>
            {
                tracing::info!("Processing {} batch request(s)", batch_requests.len());
                let workspace = #workspace_expr;
                let mut triple = BatchFileTriple::new_with_requests(batch_requests, workspace.clone())?;

                let execution_result = triple.fresh_execute(&#client_expr).await?;
                process_batch_output_and_errors::<#user_output_ty>(&*workspace, &execution_result, expected_content_type).await?;
                triple.move_all_to_done().await?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test_generate_impl_process_batch_requests {
    use super::*;

    #[traced_test]
    fn generates_impl_process_batch_requests_properly() {
        info!("Starting generates_impl_process_batch_requests_properly test.");

        // We'll supply the required fields plus batch_client_field & batch_workspace_field,
        // so the subroutine uses them for process_batch_requests logic.
        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .batch_client_field(Some(parse_quote! { some_client }))
            .batch_workspace_field(Some(parse_quote! { some_workspace }))
            // skip optional pbo/pbe
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyErr }))
            .build()
            .unwrap();

        let tokens = generate_impl_process_batch_requests(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        // Instead of looking for "process_batch_output_and_errors ("
        // we just look for the function name "process_batch_output_and_errors".
        assert!(
            code.contains("process_batch_output_and_errors"),
            "Should call process_batch_output_and_errors."
        );

        // We can still confirm that we do eventually return `Ok(())`.
        assert!(
            code.contains("Ok (())"),
            "Should return Ok(()) at the end."
        );
    }
}
