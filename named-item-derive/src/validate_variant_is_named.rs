// ---------------- [ File: src/validate_variant_is_named.rs ]
crate::ix!();

/// Validates that the variant uses named fields, else returns an error.
pub fn validate_variant_is_named<'a>(
    variant: &'a syn::Variant,
    enum_ident: &syn::Ident
) -> syn::Result<&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {

    match &variant.fields {
        syn::Fields::Named(f) => Ok(&f.named),
        _ => {
            error!(
                "Enum variant '{}' in '{}' must have named fields to use NamedItem",
                variant.ident, enum_ident
            );
            Err(syn::Error::new_spanned(
                variant,
                "NamedItem for enums requires named fields in each variant.",
            ))
        }
    }
}
