// ---------------- [ File: src/send_sync.rs ]
crate::ix!();

/// Generate `unsafe impl Send` and `unsafe impl Sync`.
pub fn generate_impl_send_sync(parsed: &LmbwParsedInput) -> TokenStream2 {
    trace!("generate_impl_send_sync: start.");

    let struct_ident = parsed.struct_ident();
    let (impl_generics, ty_generics, where_clause) = parsed.generics().split_for_impl();

    quote! {
        unsafe impl #impl_generics Send for #struct_ident #ty_generics #where_clause {}
        unsafe impl #impl_generics Sync for #struct_ident #ty_generics #where_clause {}
    }
}

#[cfg(test)]
mod test_generate_impl_send_sync {
    use super::*;

    #[traced_test]
    fn generates_send_sync() {
        info!("Starting generates_send_sync test.");

        // We are testing the generate_impl_send_sync subroutine in isolation,
        // so let's build a minimal LmbwParsedInput that includes all required
        // fields so it doesn't fail the builder. We'll fill them with dummy
        // values because for `send_sync`, we only care about struct name & generics.
        //
        //   batch_client_field => Some dummy ident
        //   batch_workspace_field => Some dummy ident
        //   model_type_field => Some dummy ident
        //   custom_error_type => Some dummy type

        let parsed = LmbwParsedInputBuilder::default()
            .struct_ident::<syn::Ident>(parse_quote! { Dummy })
            .generics(syn::Generics::default())
            .batch_client_field(Some(parse_quote! { some_client }))
            .batch_workspace_field(Some(parse_quote! { some_workspace }))
            .model_type_field(Some(parse_quote! { mt }))
            .custom_error_type(Some(parse_quote! { MyCustomError }))
            .system_message_field(Some(parse_quote! { sm }))
            .build()
            .unwrap();

        let tokens = generate_impl_send_sync(&parsed);
        let code = tokens.to_string();
        info!("Generated code: {}", code);

        assert!(code.contains("unsafe impl Send for Dummy"), "Should declare unsafe impl Send for Dummy.");
        assert!(code.contains("unsafe impl Sync for Dummy"), "Should declare unsafe impl Sync for Dummy.");
    }
}
