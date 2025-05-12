crate::ix!();

/// Builds the schema when the field is `Option<T>`. We treat `T` as not required.
pub fn build_option_schema(
    inner_ty: &syn::Type,
    doc_str: &str
) -> Option<proc_macro2::TokenStream> {
    trace!("build_option_schema => T: {:?}", inner_ty);
    let child = classify_field_type_with_justification(inner_ty, doc_str, false)?;
    Some(quote::quote!({
        #child
    }))
}
