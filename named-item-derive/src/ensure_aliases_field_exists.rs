// ---------------- [ File: src/ensure_aliases_field_exists.rs ]
crate::ix!();

/// Ensures that `aliases: Vec<String>` is present if `aliases=true`.
pub fn ensure_aliases_field_exists(
    variant: &syn::Variant,
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    _enum_ident: &syn::Ident
) -> syn::Result<()> {
    let var_ident = &variant.ident;
    let alias_field = fields_named.iter().find(|f| {
        f.ident.as_ref().map(|id| id == "aliases").unwrap_or(false)
    });
    if alias_field.is_none() {
        error!("Variant '{}' missing 'aliases' while aliases=true", var_ident);
        return Err(syn::Error::new_spanned(
            variant,
            format!("Enum variant '{}' requires `aliases: Vec<String>`", var_ident),
        ));
    }
    Ok(())
}
