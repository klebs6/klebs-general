// ---------------- [ File: src/get_language_model_client.rs ]
crate::ix!();

/// Generate `impl GetLanguageModelClient<E>`, using the user-specified error type
/// from the `#[batch_error_type(...)]` struct-level attribute. If omitted, default
/// to `OpenAIClientError`.
pub fn generate_impl_get_language_model_client(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_get_language_model_client: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    // If user provided a custom error, use it; otherwise default to OpenAIClientError
    let error_type = match &parsed.custom_error_type() {
        Some(t) => quote! { #t },
        None    => quote! { OpenAIClientError },
    };

    if let Some(c) = &parsed.batch_client_field() {
        quote! {
            impl #impl_generics GetLanguageModelClient<#error_type> for #struct_ident #ty_generics #where_clause {
                fn language_model_client(&self) -> ::std::sync::Arc<dyn LanguageModelClientInterface<#error_type>> {
                    tracing::trace!("Returning language model client Arc for our custom error type.");
                    self.#c.clone()
                }
            }
        }
    } else {
        quote! {}
    }
}

#[cfg(test)]
mod test_generate_impl_get_language_model_client {
    use super::*;

    #[traced_test]
    fn fails_when_no_batch_client_field() {
        info!("Starting fails_when_no_batch_client_field test for GetLanguageModelClient.");

        // Provide a struct missing `#[batch_client]`. It's required, so parse_derive_input_for_lmbw
        // should fail.

        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_workspace]
                some_workspace: (),

                #[model_type]
                mt: (),
            }
        };

        let result = parse_derive_input_for_lmbw(&ast);
        assert!(result.is_err(), 
            "Should fail parse because batch_client is missing.");
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("Missing required `#[batch_client]`"),
                "Error should mention missing #[batch_client]. Actual: {msg}"
            );
        }
    }

    #[traced_test]
    fn generates_impl_if_present() {
        info!("Starting generates_impl_if_present test for GetLanguageModelClient.");

        // We now *include* a `#[batch_error_type(OpenAIClientError)]`
        // attribute. That matches our field type (`Arc<OpenAIClientHandle>`
        // implements `LanguageModelClientInterface<OpenAIClientError>`).
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(OpenAIClientError)]
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

        let tokens = generate_impl_get_language_model_client(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        // Because the attribute is now explicitly `#[batch_error_type(OpenAIClientError)]`,
        // we expect the macro to generate:
        //   impl GetLanguageModelClient<OpenAIClientError> for Dummy { ... }
        assert!(
            code.contains("impl GetLanguageModelClient < OpenAIClientError > for Dummy"),
            "Should generate a GetLanguageModelClient impl with OpenAIClientError."
        );

        assert!(
            code.contains("self . some_client . clone ()"),
            "Should reference the correct field."
        );
    }

    #[traced_test]
    fn generates_impl_if_present_with_custom_error() {
        info!("Starting generates_impl_if_present_with_custom_error test for GetLanguageModelClient.");

        // This test specifically checks using a *custom* error
        // by making the field `Arc<dyn LanguageModelClientInterface<MyErr>>`
        // plus `#[batch_error_type(MyErr)]`.
        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: std::sync::Arc<dyn LanguageModelClientInterface<MyErr>>,
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

        let tokens = generate_impl_get_language_model_client(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        // Now we expect to see an impl referencing MyErr
        assert!(
            code.contains("impl GetLanguageModelClient < MyErr > for Dummy"),
            "Should generate a GetLanguageModelClient<MyErr> impl."
        );
        assert!(
            code.contains("self . some_client . clone ()"),
            "Should reference the correct field."
        );
    }
}
