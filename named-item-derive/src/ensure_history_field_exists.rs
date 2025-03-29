// ---------------- [ File: src/ensure_history_field_exists.rs ]
crate::ix!();

/// Ensures that `name_history: Vec<String>` is present if `history=true`.
pub fn ensure_history_field_exists(
    variant: &syn::Variant,
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    _enum_ident: &syn::Ident
) -> syn::Result<()> {
    let var_ident = &variant.ident;
    let hist_field = fields_named.iter().find(|f| {
        f.ident.as_ref().map(|id| id == "name_history").unwrap_or(false)
    });
    if hist_field.is_none() {
        error!("Variant '{}' missing 'name_history' while history=true", var_ident);
        return Err(syn::Error::new_spanned(
            variant,
            format!("Enum variant '{}' requires `name_history: Vec<String>`", var_ident),
        ));
    }
    Ok(())
}
