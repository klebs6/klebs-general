// ---------------- [ File: ai-json-template-derive/src/handle_unit_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_unit_variant(
    var_ident: &syn::Ident,
    skip_self_just: bool
) -> (
    proc_macro2::TokenStream, // variant in the Justification enum
    proc_macro2::TokenStream, // variant in the Confidence enum
    Option<String>,           // first-variant justification field name
    Option<String>            // first-variant confidence field name
)
{
    debug!(
        "Handling unit variant '{}', skip_self_just={}",
        var_ident,
        skip_self_just
    );

    if skip_self_just {
        let jvar = quote::quote! { #var_ident {} };
        let cvar = quote::quote! { #var_ident {} };
        (jvar, cvar, None, None)
    } else {
        let jvar = quote::quote! { #var_ident { variant_justification: String } };
        let cvar = quote::quote! { #var_ident { variant_confidence: f32 } };
        (
            jvar,
            cvar,
            Some("variant_justification".to_string()),
            Some("variant_confidence".to_string())
        )
    }
}
