crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn build_named_field_child_schema_expr(
    field: &syn::Field,
    doc_str: &str,
    is_required: bool,
    skip_child_just: bool,
) -> Option<proc_macro2::TokenStream> {
    tracing::trace!(
        "build_named_field_child_schema_expr: field='{}', required={}, skip_child_just={}",
        field
            .ident
            .as_ref()
            .map_or("<unnamed?>".to_string(), |i| i.to_string().to_string()),
        is_required,
        skip_child_just
    );

    classify_field_type_for_child(&field.ty, doc_str, is_required, skip_child_just)
}
