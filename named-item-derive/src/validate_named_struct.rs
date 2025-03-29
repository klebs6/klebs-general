// ---------------- [ File: src/validate_named_struct.rs ]
crate::ix!();

/// Validates that the `DataStruct` is named, returns the named fields on success.
pub fn validate_named_struct<'a>(
    ast: &'a syn::DeriveInput,
    ds: &'a syn::DataStruct,
    struct_name: &syn::Ident
) -> syn::Result<&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {
    use tracing::error;

    match &ds.fields {
        syn::Fields::Named(f) => Ok(&f.named),
        _ => {
            error!("Struct '{}' does not have named fields", struct_name);
            Err(syn::Error::new_spanned(
                ast,
                "NamedItem requires a struct with named fields.",
            ))
        }
    }
}
