// ---------------- [ File: language-model-batch-workflow-derive/src/get_batch_workspace.rs ]
crate::ix!();

pub fn generate_impl_get_batch_workspace(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_get_batch_workspace: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    // In FullBatchWorkspaceInterface<E,I>, the E is our error type, and the I is taken 
    // from <Self as ComputeLanguageModelCoreQuery>::Seed (which is the 'input' type).
    if let Some(w) = &parsed.batch_workspace_field() {
        quote! {
            impl #impl_generics GetBatchWorkspace<BatchWorkspaceError, <#struct_ident #ty_generics as ComputeLanguageModelCoreQuery>::Seed>
                for #struct_ident #ty_generics #where_clause
            {
                fn workspace(&self)
                    -> ::std::sync::Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError, <#struct_ident #ty_generics as ComputeLanguageModelCoreQuery>::Seed>>
                {
                    trace!("Returning batch workspace arc for FullBatchWorkspaceInterface with E/I generics.");
                    self.#w.clone()
                }
            }
        }
    } else {
        // If the user didn't specify a #[batch_workspace] field, generate no code.
        quote! {}
    }
}

#[cfg(test)]
mod test_generate_impl_get_batch_workspace {
    use super::*;

    #[traced_test]
    fn generates_impl_if_present() {
        info!("Starting generates_impl_if_present test for GetBatchWorkspace.");

        // Provide a struct that has the required fields/attributes:
        //   #[batch_error_type(MyErr)]
        //   #[batch_client]
        //   #[batch_workspace]
        //   #[model_type]
        //
        // Then parse it and confirm we generate an impl referencing both E and I.
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: std::sync::Arc<OpenAIClientHandle>,

                #[batch_workspace]
                some_workspace: std::sync::Arc<BatchWorkspace>,

                #[model_type]
                mt: LanguageModelType,
            }
        };

        let parsed = match parse_derive_input_for_lmbw(&ast) {
            Ok(x) => x,
            Err(e) => panic!("Expected parse to succeed but got error: {e}"),
        };

        let tokens = generate_impl_get_batch_workspace(&parsed);
        let code = tokens.to_string();
        debug!("Generated code: {}", code);

        // Now we expect an impl that supplies both the error type (MyErr) and
        // the input type (<Dummy as ComputeLanguageModelCoreQuery>::Seed).
        assert!(
            code.contains("impl GetBatchWorkspace < MyErr , < Dummy as ComputeLanguageModelCoreQuery > :: Seed > for Dummy"),
            "Should generate impl GetBatchWorkspace<MyErr, <Dummy as ComputeLanguageModelCoreQuery>::Seed> for Dummy."
        );

        // And it should clone the correct field.
        assert!(
            code.contains("self . some_workspace . clone ()"),
            "Should reference the correct field for returning the workspace Arc."
        );
    }

    #[traced_test]
    fn fails_when_no_batch_workspace_field() {
        info!("Starting fails_when_no_batch_workspace_field test for GetBatchWorkspace.");

        // Provide a struct missing the #[batch_workspace] attribute. That is required.
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: (),

                #[model_type]
                mt: (),
            }
        };

        let result = parse_derive_input_for_lmbw(&ast);
        assert!(
            result.is_err(),
            "Should fail parse because batch_workspace is missing."
        );

        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("Missing required `#[batch_workspace]`"),
                "Error should mention missing #[batch_workspace]. Actual: {msg}"
            );
        }
    }
}
