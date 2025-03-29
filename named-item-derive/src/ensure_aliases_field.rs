// ---------------- [ File: src/ensure_aliases_field.rs ]
crate::ix!();

/// Ensures that `aliases: Vec<String>` is present if `aliases=true`.
pub fn ensure_aliases_field(
    ast: &syn::DeriveInput,
    named_fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_name: &syn::Ident
) -> syn::Result<()> {

    let alias_field = named_fields.iter().find(|field| {
        field.ident.as_ref().map(|id| id == "aliases").unwrap_or(false)
    });
    if alias_field.is_none() {
        error!("aliases=true but 'aliases' field not found for '{}'", struct_name);
        return Err(syn::Error::new_spanned(
            ast,
            "aliases=true but no `aliases: Vec<String>` field found.",
        ));
    }

    Ok(())
}
