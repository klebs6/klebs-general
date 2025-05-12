crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn gather_schemas_and_placeholders_for_named_fields(
    fields: &syn::FieldsNamed
) -> Vec<proc_macro2::TokenStream> {
    use tracing::debug;

    let mut expansions = Vec::new();
    for field in &fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                debug!("Skipping unnamed field in a named struct?");
                continue;
            }
        };
        let field_name_str = field_ident.to_string();
        debug!("Processing field '{}'", field_name_str);

        let doc_str = gather_doc_comments(&field.attrs).join("\n");
        let is_required = extract_option_inner(&field.ty).is_none();
        let skip_self_just   = is_justification_disabled_for_field(field);
        let skip_child_just  = skip_self_just || is_justification_disabled_for_inner(field);

        // (A) Normal child schema
        if let Some(child_expr) = build_named_field_child_schema_expr(field, &doc_str, is_required, skip_child_just) {
            expansions.push(quote::quote! {
                map.insert(#field_name_str.to_string(), #child_expr);
            });
        }

        // (B) Just/conf placeholders if skip_self_just is false
        if !skip_self_just {
            let just_conf_ts = build_named_field_just_conf_placeholders(&field_name_str);
            expansions.push(just_conf_ts);
        }
    }
    expansions
}
