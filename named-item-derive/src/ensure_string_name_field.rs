// ---------------- [ File: src/ensure_string_name_field.rs ]
crate::ix!();

/// Ensures that the struct has a `name: String` field.
pub fn ensure_string_name_field(
    ast:          &syn::DeriveInput,
    named_fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_name:  &syn::Ident

) -> syn::Result<()> {

    let name_field = named_fields.iter().find(|field| {
        field.ident.as_ref().map(|id| id == "name").unwrap_or(false)
    });

    if name_field.is_none() {
        error!("No 'name' field found in struct '{}'", struct_name);
        return Err(syn::Error::new_spanned(
            ast,
            "Struct must have `name: String`.",
        ));
    }

    let name_ty = &name_field.unwrap().ty;

    let is_string = match name_ty {
        syn::Type::Path(tp) => {
            tp.path.segments.last().map(|seg| seg.ident == "String").unwrap_or(false)
        }
        _ => false,
    };

    if !is_string {
        error!("Field 'name' in '{}' is not of type 'String'", struct_name);
        return Err(syn::Error::new_spanned(
            name_ty,
            "`name` field must be `String`",
        ));
    }

    Ok(())
}
