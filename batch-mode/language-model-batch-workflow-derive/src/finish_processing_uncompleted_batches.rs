// ---------------- [ File: src/finish_processing_uncompleted_batches.rs ]
crate::ix!();

/// Generate the `impl FinishProcessingUncompletedBatches` code.
pub fn generate_impl_finish_processing_uncompleted_batches(parsed: &LmbwParsedInput) -> TokenStream2 {
    tracing::trace!("generate_impl_finish_processing_uncompleted_batches: start.");

    let struct_ident = parsed.struct_ident();
    let struct_name_str = struct_ident.to_string();

    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    // The user’s custom error type (e.g. `MyErr`) or our fallback
    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { TokenExpanderError },
    };

    // The user's chosen JSON format type or fallback to CamelCaseTokenWithComment
    let user_output_ty = match parsed.json_output_format_type() {
        Some(t) => quote! { #t },
        None    => quote! { CamelCaseTokenWithComment },
    };

    // We create identifiers for the new bridging function + pointer
    // specialized to this struct’s chosen type. For instance, if the
    // struct is named `MyValidStruct`, we generate:
    //   fn MyValidStruct_output_file_bridge_fn<'a>(...) -> ...
    //   pub const MYVALIDSTRUCT_OUTPUT_FILE_BRIDGE: ...
    //
    // We'll do a simple transform to uppercase the struct name for the const.
    let bridge_fn_ident = syn::Ident::new(
        &format!("{}_output_file_bridge_fn", struct_name_str),
        struct_ident.span(),
    );
    let bridge_const_ident = syn::Ident::new(
        &format!("{}_OUTPUT_FILE_BRIDGE", struct_name_str.to_ascii_uppercase()),
        struct_ident.span(),
    );

    // If the user gave us a custom bridging field via #[custom_process_batch_output_fn],
    // we do not use the specialized function pointer. Instead, we reference that field.
    let output_fn_expr = if let Some(custom_out) = &parsed.process_batch_output_fn_field() {
        quote! { &self.#custom_out }
    } else {
        // Otherwise, we pass `&MYSTRUCT_OUTPUT_FILE_BRIDGE`.
        quote! { &#bridge_const_ident }
    };

    // If the user gave us a custom bridging field for error handling, use it;
    // else fallback to PROCESS_ERROR_FILE_BRIDGE.
    let error_fn_expr = if let Some(err_fn) = &parsed.process_batch_error_fn_field() {
        quote! { &self.#err_fn }
    } else {
        quote! { &PROCESS_ERROR_FILE_BRIDGE }
    };

    // The expressions for `workspace` and `client` references
    let workspace_expr = if let Some(w) = &parsed.batch_workspace_field() {
        quote! { self.#w.clone() }
    } else {
        quote! { self.workspace() }
    };
    let client_expr = if let Some(c) = &parsed.batch_client_field() {
        quote! { self.#c.clone() }
    } else {
        quote! { self.client() }
    };

    // We generate code in two parts:
    //   1) A specialized bridging function + const pointer: 
    //      <StructName>_output_file_bridge_fn + <STRUCTNAME>_OUTPUT_FILE_BRIDGE
    //   2) The `impl FinishProcessingUncompletedBatches for <Struct>` block

    let specialized_bridge_def = quote! {
        /// A specialized bridging function for `#struct_ident`, using type
        /// `#user_output_ty` for JSON expansions if needed.
        fn #bridge_fn_ident<'a>(
            triple: &'a BatchFileTriple,
            workspace: &'a (dyn BatchWorkspaceInterface + 'a),
            ect: &'a ExpectedContentType,
        ) -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Result<(), BatchOutputProcessingError>> + ::std::marker::Send + 'a>>
        {
            Box::pin(async move {
                process_output_file::<#user_output_ty>(triple, workspace, ect).await
            })
        }

        /// A public constant pointer to that bridging function, matching `BatchWorkflowProcessOutputFileFn`.
        pub const #bridge_const_ident: BatchWorkflowProcessOutputFileFn = #bridge_fn_ident;
    };

    // The actual impl block
    let finish_impl = quote! {
        #[async_trait]
        impl #impl_generics FinishProcessingUncompletedBatches for #struct_ident #ty_generics #where_clause {
            type Error = #error_type;

            async fn finish_processing_uncompleted_batches(
                &self,
                expected_content_type: &ExpectedContentType
            ) -> Result<(), Self::Error>
            {
                // We'll do a local import so we can reference
                //   PROCESS_ERROR_FILE_BRIDGE, and also confirm that
                //   `process_output_file` is in scope.
                use batch_mode_batch_workflow::{
                    PROCESS_ERROR_FILE_BRIDGE,
                    BatchWorkflowProcessOutputFileFn,
                };

                tracing::info!("Finishing uncompleted batches if any remain for {}.", stringify!(#struct_ident));
                let workspace = #workspace_expr;
                let language_model_client = #client_expr;

                let mut batch_triples = workspace.clone().gather_all_batch_triples().await?;
                tracing::info!("Reconciling unprocessed batch files in the work directory for {}.", stringify!(#struct_ident));

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
    };

    // Combine them into one chunk of expanded code
    quote! {
        // Specialized bridging items for #struct_ident:
        #specialized_bridge_def

        // The actual FinishProcessingUncompletedBatches impl:
        #finish_impl
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

        // We now check for the local bridging function you inject:
        assert!(
            code.contains("& DUMMY_OUTPUT_FILE_BRIDGE"),
            "Should fallback to a local bridging function for the output file."
        );
        assert!(
            code.contains("& PROCESS_ERROR_FILE_BRIDGE"),
            "Should fallback to PROCESS_ERROR_FILE_BRIDGE for the error file bridging function."
        );
    }
}
