crate::ix!();

pub fn flatten_named_variant_fields(
    named_fields:             &syn::FieldsNamed,
    skip_field_self_just_fn: impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:         impl Fn(&syn::Type) -> bool,
    skip_child_just:         bool,
    flatten_named_field_fn:  impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
) -> FlattenedFieldResult
{
    trace!(
        "flatten_named_variant_fields: {} field(s) to process.",
        named_fields.named.len()
    );

    let mut field_decls_for_fields  = Vec::new();
    let mut pattern_vars_for_fields = Vec::new();
    let mut item_inits              = Vec::new();
    let mut just_inits_for_fields   = Vec::new();
    let mut conf_inits_for_fields   = Vec::new();

    for field in &named_fields.named {
        let f_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Unnamed field in 'named' variant? Skipping.");
                continue;
            }
        };

        let skip_f_self = skip_field_self_just_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (decls, i_init, j_init, c_init) =
            flatten_named_field_fn(f_ident, &field.ty, skip_f_self, child_skip);

        // Insert them (with commas after the first).
        for (i, decl) in decls.into_iter().enumerate() {
            if i == 0 {
                field_decls_for_fields.push(decl);
            } else {
                let with_comma = quote::quote! { #decl, };
                field_decls_for_fields.push(with_comma);
            }
        }

        pattern_vars_for_fields.push(quote::quote! { #f_ident });

        if !i_init.is_empty() {
            item_inits.push(i_init);
        }
        if !j_init.is_empty() {
            just_inits_for_fields.push(j_init);
        }
        if !c_init.is_empty() {
            conf_inits_for_fields.push(c_init);
        }
    }

    FlattenedFieldResult {
        field_decls_for_fields,
        pattern_vars_for_fields,
        item_inits,
        just_inits_for_fields,
        conf_inits_for_fields
    }
}
