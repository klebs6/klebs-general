// ---------------- [ File: src/create_variant_pattern_and_bindings.rs ]
crate::ix!();

/// Creates the variant pattern and identifies the name, history, and alias bindings.
pub fn create_variant_pattern_and_bindings(
    enum_ident: &syn::Ident,
    var_ident: &syn::Ident,
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>
) -> (
    proc_macro2::TokenStream,        // pattern
    Option<syn::Ident>,              // name_binding
    Option<syn::Ident>,              // history_binding
    Option<syn::Ident>,              // alias_binding
) {
    let mut variant_pattern = Vec::new();
    let mut name_binding = None;
    let mut hist_binding = None;
    let mut alias_binding = None;

    for f in fields_named {
        let id = f.ident.as_ref().unwrap().clone();
        let id_str = id.to_string();
        variant_pattern.push(quote! { #id });
        match id_str.as_str() {
            "name" => name_binding = Some(id),
            "name_history" => hist_binding = Some(id),
            "aliases" => alias_binding = Some(id),
            _ => {}
        }
    }

    let pat = quote! {
        #enum_ident::#var_ident { #(#variant_pattern),* }
    };

    (pat, name_binding, hist_binding, alias_binding)
}
