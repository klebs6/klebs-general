// ---------------- [ File: src/ensure_name_history_field.rs ]
crate::ix!();

/// Ensures that `name_history: Vec<String>` is present if `history=true`.
pub fn ensure_name_history_field(
    ast: &syn::DeriveInput,
    named_fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_name: &syn::Ident
) -> syn::Result<()> {
    use tracing::error;

    let hist_field = named_fields.iter().find(|field| {
        field.ident.as_ref().map(|id| id == "name_history").unwrap_or(false)
    });
    if hist_field.is_none() {
        error!("history=true but 'name_history' field not found for '{}'", struct_name);
        return Err(syn::Error::new_spanned(
            ast,
            "history=true but no `name_history: Vec<String>` field found.",
        ));
    }

    Ok(())
}
