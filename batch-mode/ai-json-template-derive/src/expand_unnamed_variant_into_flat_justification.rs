// ---------------- [ File: ai-json-template-derive/src/expand_unnamed_variant_into_flat_justification.rs ]
crate::ix!();

/// Main entry point: expands an **unnamed (tuple) variant** into “flat justification” form,
/// with no hidden special-case hacks. All variants follow the same logic flow.
pub fn expand_unnamed_variant_into_flat_justification(
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    unnamed_fields:      &FieldsUnnamed,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    skip_self_just:      bool,
    skip_child_just:     bool,
    flatten_unnamed_field_fn: impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
    skip_field_self_just_fn:  impl Fn(&Field) -> bool,
    is_leaf_type_fn:          impl Fn(&syn::Type) -> bool
) -> (TokenStream2, TokenStream2)
{
    trace!(
        "Expanding unnamed variant '{}' in enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    // 1) Gather expansions from top-level justification/conf plus each field
    let expansions = gather_unnamed_variant_expansions(
        parent_enum_ident,
        variant_ident,
        unnamed_fields,
        skip_self_just,
        skip_child_just,
        &flatten_unnamed_field_fn,
        &skip_field_self_just_fn,
        &is_leaf_type_fn,
    );

    // 2) Build the final flattened variant (the enum definition side)
    let flat_variant_ts = finalize_flat_unnamed_variant_ts(variant_ident, &expansions);

    // 3) Build the final from-arm match (the impl From<FlatJustifiedFooEnum> side)
    let from_arm_ts = finalize_from_arm_unnamed_variant_ts(
        parent_enum_ident,
        variant_ident,
        justification_ident,
        confidence_ident,
        &expansions,
    );

    (flat_variant_ts, from_arm_ts)
}

#[cfg(test)]
mod test_expand_unnamed_variant_into_flat_justification {
    use super::*;

    #[traced_test]
    fn test_two_tuple_fields() {
        // We'll build an unnamed variant: (bool, String)
        let f0 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { bool },
            mutability: FieldMutability::None,
        };
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { String },
            mutability: FieldMutability::None,
        };
        let fields_unnamed = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f0);
                p.push(f1);
                p
            },
        };

        fn dummy_flatten_unnamed_field(
            field_ident: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            // Pretend we produce a single field in the flattened variant with the same name
            let decl = quote! { #field_ident: #field_ident, };
            (vec![ quote!{ #decl } ], quote!{ #field_ident }, TokenStream2::new(), TokenStream2::new())
        }
        fn dummy_skip_field(_f: &syn::Field) -> bool { false }
        fn dummy_is_leaf_type(_t: &syn::Type) -> bool { false }

        let parent = Ident::new("SomeEnum", proc_macro2::Span::call_site());
        let var = Ident::new("TupleVar", proc_macro2::Span::call_site());
        let just_id = Ident::new("SomeEnumJustification", proc_macro2::Span::call_site());
        let conf_id = Ident::new("SomeEnumConfidence", proc_macro2::Span::call_site());

        let (fv, arm) = expand_unnamed_variant_into_flat_justification(
            &parent,
            &var,
            &fields_unnamed,
            &just_id,
            &conf_id,
            /*skip_self_just=*/ false,
            /*skip_child_just=*/ false,
            dummy_flatten_unnamed_field,
            dummy_skip_field,
            dummy_is_leaf_type
        );

        let fv_str = fv.to_string();
        let arm_str = arm.to_string();

        // The variant def should mention "enum_variant_justification" plus "f0" and "f1".
        assert!(fv_str.contains("TupleVar {"));
        assert!(fv_str.contains("enum_variant_justification"));
        assert!(fv_str.contains("f0 : f0"));
        assert!(fv_str.contains("f1 : f1"));

        // The from arm pattern => 
        //   FlatJustifiedSomeEnum::TupleVar { enum_variant_justification, ..., f0, f1 } => ...
        assert!(arm_str.contains("FlatJustifiedSomeEnum :: TupleVar"));
        assert!(arm_str.contains("f0 , f1"));
        assert!(arm_str.contains("SomeEnum :: TupleVar ( f0 , f1 )"));
    }

    fn dummy_flatten_unnamed_field(
        fid: &syn::Ident,
        _ty: &Type,
        _skip_self: bool,
        _skip_child: bool
    ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
        // for demonstration, produce a single field decl plus direct usage
        let decl = quote! { #fid : #fid };
        let item_init = quote! { #fid };
        let just_init = quote! { #fid_just };
        let conf_init = quote! { #fid_conf };
        (vec![decl], item_init, just_init, conf_init)
    }
    fn dummy_skip_field(_f: &Field) -> bool { false }
    fn dummy_is_leaf(_t: &Type) -> bool { false }

    #[traced_test]
    fn test_expand_unnamed_variant_no_special_hacks() {
        let f0 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { bool },
            mutability: FieldMutability::None,
        };
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { String },
            mutability: FieldMutability::None,
        };
        let fields = FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f0);
                p.push(f1);
                p
            },
        };

        let parent = syn::Ident::new("MyEnum", Span::call_site());
        let var = syn::Ident::new("SomeTupleVariant", Span::call_site());
        let just = syn::Ident::new("MyEnumJustification", Span::call_site());
        let conf = syn::Ident::new("MyEnumConfidence", Span::call_site());

        let (fv, arm) = expand_unnamed_variant_into_flat_justification(
            &parent,
            &var,
            &fields,
            &just,
            &conf,
            /*skip_self_just=*/ false,
            /*skip_child_just=*/ false,
            dummy_flatten_unnamed_field,
            dummy_skip_field,
            dummy_is_leaf
        );

        let fv_str = fv.to_string();
        let arm_str = arm.to_string();

        // We expect a variant snippet: "SomeTupleVariant { enum_variant_justification, enum_variant_confidence, f0: f0, f1: f1 },"
        assert!(fv_str.contains("SomeTupleVariant {"));
        assert!(fv_str.contains("enum_variant_justification"));
        assert!(fv_str.contains("enum_variant_confidence"));
        assert!(fv_str.contains("f0 : f0"));
        assert!(fv_str.contains("f1 : f1"));

        // The from-arm => "FlatJustifiedMyEnum :: SomeTupleVariant { enum_variant_justification, ..., f0, f1 } => { ... }"
        // etc. This is the normal approach, no special hack.
        assert!(arm_str.contains("FlatJustifiedMyEnum :: SomeTupleVariant {"));
        assert!(arm_str.contains("f0 , f1"));
        assert!(arm_str.contains("MyEnum :: SomeTupleVariant ( f0 , f1 )"));
        assert!(arm_str.contains("MyEnumJustification :: SomeTupleVariant"));
        assert!(arm_str.contains("MyEnumConfidence :: SomeTupleVariant"));
    }
}
