// ---------------- [ File: ai-json-template-derive/src/gather_unnamed_variant_expansions.rs ]
crate::ix!();

pub fn gather_unnamed_variant_expansions(
    parent_enum_ident:         &syn::Ident,
    variant_ident:             &syn::Ident,
    unnamed_fields:            &FieldsUnnamed,
    skip_self_just:           bool,
    skip_child_just:          bool,
    flatten_unnamed_field_fn: &impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
    skip_field_self_just_fn:  &impl Fn(&Field) -> bool,
    is_leaf_type_fn:          &impl Fn(&syn::Type) -> bool,
) -> UnnamedVariantExpansion {
    debug!(
        "Gathering expansions for unnamed variant '{}::{}'",
        parent_enum_ident,
        variant_ident
    );

    let mut expansions = UnnamedVariantExpansionBuilder::default()
        .field_declarations(vec![])
        .pattern_vars(vec![])
        .item_exprs(vec![])
        .just_vals(vec![])
        .conf_vals(vec![])
        .build()
        .unwrap();

    // Top-level justification/conf if not skipped
    if !skip_self_just {
        expansions.field_declarations_mut().push(quote! {
            #[serde(default)]
            enum_variant_justification:String
        });
        expansions.field_declarations_mut().push(quote! {
            #[serde(default)]
            enum_variant_confidence:f32
        });
        expansions.pattern_vars_mut().push(quote! { enum_variant_justification });
        expansions.pattern_vars_mut().push(quote! { enum_variant_confidence });
        expansions.just_vals_mut().push(quote! { variant_justification: enum_variant_justification });
        expansions.conf_vals_mut().push(quote! { variant_confidence: enum_variant_confidence });
    }

    // Now each unnamed field
    for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
        let field_ident = syn::Ident::new(&format!("f{}", idx), field.span());

        let skip_f_self = skip_field_self_just_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (field_decls, i_init, j_init, c_init) =
            flatten_unnamed_field_fn(&field_ident, &field.ty, skip_f_self, child_skip);

        expansions.field_declarations_mut().extend(field_decls);
        expansions.pattern_vars_mut().push(quote! { #field_ident });

        if !i_init.is_empty() {
            expansions.item_exprs_mut().push(i_init);
        }
        if !j_init.is_empty() {
            expansions.just_vals_mut().push(j_init);
        }
        if !c_init.is_empty() {
            expansions.conf_vals_mut().push(c_init);
        }
    }

    expansions
}
