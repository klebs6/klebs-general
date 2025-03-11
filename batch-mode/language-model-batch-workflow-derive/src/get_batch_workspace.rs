// ---------------- [ File: src/get_batch_workspace.rs ]
crate::ix!();

/// Generate `impl GetBatchWorkspace<BatchWorkspaceError>`.
/// We rely on the `#[batch_workspace]`-annotated field if available.
/// Otherwise, we generate nothing.
pub fn generate_impl_get_batch_workspace(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_get_batch_workspace: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    if let Some(w) = &parsed.batch_workspace_field() {
        quote! {
            impl #impl_generics GetBatchWorkspace<BatchWorkspaceError> for #struct_ident #ty_generics #where_clause {
                fn workspace(&self) -> ::std::sync::Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>> {
                    self.#w.clone()
                }
            }
        }
    } else {
        quote! {}
    }
}

#[cfg(test)]
mod test_generate_impl_get_batch_workspace {
    use super::*;

    #[traced_test]
    fn generates_impl_if_present() {
        info!("Starting generates_impl_if_present test for GetBatchWorkspace.");

        // Provide a struct that has the `#[batch_workspace]` attribute plus all other
        // required attributes:
        //
        //   - #[batch_client]
        //   - #[batch_workspace]
        //   - #[model_type]
        //   - #[batch_error_type(MyErr)]
        //
        // Then parse it fully with parse_derive_input_for_lmbw, ensuring we succeed,
        // and check that the subroutine for get_batch_workspace generates code referencing
        // the correct field.

        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: std::sync::Arc<OpenAIClientHandle>,
                #[batch_workspace]
                some_workspace: std::sync::Arc<BatchWorkspace>,
                #[system_message]
                ect: String,
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
        info!("Generated code: {}", code);

        assert!(code.contains("impl GetBatchWorkspace < BatchWorkspaceError > for Dummy"),
                "Should generate a GetBatchWorkspace impl.");
        assert!(code.contains("self . some_workspace . clone ()"),
                "Should reference the correct field.");
    }

    #[traced_test]
    fn fails_when_no_batch_workspace_field() {
        info!("Starting fails_when_no_batch_workspace_field test for GetBatchWorkspace.");

        // Provide a struct missing the `#[batch_workspace]` attribute. This is required,
        // so parse_derive_input_for_lmbw should fail with an error, and we never even
        // get to the code-gen subroutine.

        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: (),

                #[system_message]
                ect: String,

                #[model_type]
                mt: (),
            }
        };

        let result = parse_derive_input_for_lmbw(&ast);
        assert!(result.is_err(), 
            "Should fail parse because batch_workspace is missing.");
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("Missing required `#[batch_workspace]`"),
                "Error should mention missing #[batch_workspace]. Actual: {msg}"
            );
        }
    }
}
