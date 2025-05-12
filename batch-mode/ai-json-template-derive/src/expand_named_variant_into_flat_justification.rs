// ---------------- [ File: ai-json-template-derive/src/expand_named_variant_into_flat_justification.rs ]
crate::ix!();

pub fn expand_named_variant_into_flat_justification(
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    named_fields:        &syn::FieldsNamed,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    skip_self_just:      bool,
    skip_child_just:     bool,
    flatten_named_field_fn: impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream),
    skip_field_self_just_fn: impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:         impl Fn(&syn::Type) -> bool
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    trace!(
        "expand_named_variant_into_flat_justification: enum='{}', variant='{}', skip_self_just={}, skip_child_just={}",
        parent_enum_ident,
        variant_ident,
        skip_self_just,
        skip_child_just
    );

    // 2) Otherwise, handle the normal logic
    let flat_parent_ident = syn::Ident::new(
        &format!("FlatJustified{}", parent_enum_ident),
        parent_enum_ident.span()
    );
    let renamed_just_var = rename_unit_to_unitvariant(variant_ident);

    // Step A) Possibly insert top-level enum justification/conf fields.
    let top_level_just_result = build_top_level_just_fields_for_variant(variant_ident, skip_self_just);

    // Step B) Flatten each named field
    let flattened_field_result = flatten_named_variant_fields(
        &named_fields,
        skip_field_self_just_fn,
        is_leaf_type_fn,
        skip_child_just,
        &flatten_named_field_fn,
    );

    // Step C) Construct the final flat variant snippet
    let flat_variant_ts = build_flat_variant_snippet_named(
        variant_ident,
        top_level_just_result.field_decls_top(),
        flattened_field_result.field_decls_for_fields()
    );

    // Step D) Build the final “from” arm snippet
    let from_arm_ts = build_from_arm_for_named(
        &flat_parent_ident,
        parent_enum_ident,
        variant_ident,
        &renamed_just_var,
        justification_ident,
        confidence_ident,
        top_level_just_result.pattern_vars_top(),
        flattened_field_result.pattern_vars_for_fields(),
        top_level_just_result.just_inits_top(),
        flattened_field_result.just_inits_for_fields(),
        top_level_just_result.conf_inits_top(),
        flattened_field_result.conf_inits_for_fields(),
        flattened_field_result.item_inits()
    );

    (flat_variant_ts, from_arm_ts)
}

#[cfg(test)]
mod test_expand_named_variant_into_flat_justification {
    use super::*;

    #[traced_test]
    fn test_simple_named_variant() {
        // We'll build a FieldsNamed with two fields: "alpha: u8" and "beta: String"
        let alpha = Field {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            ident: Some(Ident::new("alpha", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: syn::parse_quote! { u8 },
            mutability: FieldMutability::None,
        };
        let beta = Field {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            ident: Some(Ident::new("beta", proc_macro2::Span::call_site())),
            colon_token: Some(Default::default()),
            ty: syn::parse_quote! { String },
            mutability: FieldMutability::None,
        };
        let fields_named = FieldsNamed {
            brace_token: Default::default(),
            named: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(alpha);
                p.push(beta);
                p
            },
        };

        fn dummy_flatten_named_field(
            field_ident: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            // For testing, pretend we produce one flattened field with same name, 
            // plus no justification/conf expansions. 
            // That’s enough to test this subroutine’s logic of pattern matching.
            let decl = quote! { #field_ident: #field_ident, };
            let i_init = quote! { #field_ident };
            (vec! [ quote!{ #decl } ], i_init, TokenStream2::new(), TokenStream2::new())
        }
        fn dummy_skip_field(_f: &syn::Field) -> bool { false }
        fn dummy_is_leaf_type(_t: &syn::Type) -> bool { false }

        let parent = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let var = Ident::new("StructVariant", proc_macro2::Span::call_site());
        let just_id = Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf_id = Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());

        let (fv, arm) = expand_named_variant_into_flat_justification(
            &parent,
            &var,
            &fields_named,
            &just_id,
            &conf_id,
            /*skip_self_just=*/false,
            /*skip_child_just=*/false,
            dummy_flatten_named_field,
            dummy_skip_field,
            dummy_is_leaf_type
        );

        let fv_str = fv.to_string();
        let arm_str = arm.to_string();

        // We expect fv_str to define "StructVariant { ... }" 
        // with "enum_variant_justification" and "enum_variant_confidence" plus "alpha", "beta".
        assert!(fv_str.contains("StructVariant {"));
        assert!(fv_str.contains("enum_variant_justification"));
        assert!(fv_str.contains("alpha: alpha"));
        assert!(fv_str.contains("beta: beta"));

        // The from arm should do: 
        //   FlatJustifiedMyEnum::StructVariant { enum_variant_justification, enum_variant_confidence, alpha, beta } => ...
        //   item: MyEnum::StructVariant { alpha: alpha, beta: beta }, 
        //   justification: MyEnumJustification::StructVariant { variant_justification: enum_variant_justification }, 
        //   confidence: MyEnumConfidence::StructVariant { variant_confidence: enum_variant_confidence }, ...
        assert!(arm_str.contains("FlatJustifiedMyEnum :: StructVariant"));
        assert!(arm_str.contains("alpha , beta"));
        assert!(arm_str.contains("MyEnum::StructVariant"));
        assert!(arm_str.contains("alpha : alpha"));
        assert!(arm_str.contains("beta : beta"));
        assert!(arm_str.contains("variant_justification : enum_variant_justification"));
    }
}
