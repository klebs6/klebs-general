// ---------------- [ File: src/get_language_model_client.rs ]
crate::ix!();

/// Generate `impl GetLanguageModelClient<OpenAIClientError>`.
/// We rely on the `#[batch_client]`-annotated field if available.
/// Otherwise, we generate nothing.
pub fn generate_impl_get_language_model_client(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_get_language_model_client: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    if let Some(c) = &parsed.batch_client_field() {
        quote! {
            impl #impl_generics GetLanguageModelClient<OpenAIClientError> for #struct_ident #ty_generics #where_clause {
                fn language_model_client(&self) -> ::std::sync::Arc<dyn LanguageModelClientInterface<OpenAIClientError>> {
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
    fn generates_impl_if_present() {
        info!("Starting generates_impl_if_present test for GetLanguageModelClient.");

        // Provide a struct that has `#[batch_client]` plus other required attributes,
        // then confirm the subroutine references that field.

        let ast: DeriveInput = parse_quote! {
            #[batch_error_type(MyErr)]
            struct Dummy {
                #[batch_client]
                some_client: (),

                #[batch_workspace]
                some_workspace: (),

                #[expected_content_type]
                ect: (),
                #[model_type]
                mt: (),
            }
        };

        let parsed = match parse_derive_input_for_lmbw(&ast) {
            Ok(x) => x,
            Err(e) => panic!("Expected parse to succeed but got error: {e}"),
        };

        let tokens = generate_impl_get_language_model_client(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        assert!(code.contains("impl GetLanguageModelClient < OpenAIClientError > for Dummy"),
                "Should generate a GetLanguageModelClient impl.");
        assert!(code.contains("self . some_client . clone ()"),
                "Should reference the correct field.");
    }

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

                #[expected_content_type]
                ect: (),
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
}

