// ---------------- [ File: src/ensure_name_field_exists.rs ]
crate::ix!();

/// Ensures that `name: String` is present.
pub fn ensure_name_field_exists(
    variant: &syn::Variant,
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    _enum_ident: &syn::Ident
) -> syn::Result<()> {
    let var_ident = &variant.ident;
    let name_field = fields_named.iter().find(|f| {
        f.ident.as_ref().map(|id| id == "name").unwrap_or(false)
    });
    if name_field.is_none() {
        error!("Variant '{}' missing 'name: String'", var_ident);
        return Err(syn::Error::new_spanned(
            variant,
            format!("Enum variant '{}' must have `name: String` field", var_ident),
        ));
    }

    // Confirm name is actually a String
    let is_string = match &name_field.unwrap().ty {
        syn::Type::Path(tp) => {
            tp.path
              .segments
              .last()
              .map(|seg| seg.ident == "String")
              .unwrap_or(false)
        }
        _ => false,
    };
    if !is_string {
        error!("Variant '{}' has 'name' field but not type 'String'", var_ident);
        return Err(syn::Error::new_spanned(
            &name_field.unwrap().ty,
            format!("Variant '{}' `name` field must be `String`", var_ident),
        ));
    }

    Ok(())
}
