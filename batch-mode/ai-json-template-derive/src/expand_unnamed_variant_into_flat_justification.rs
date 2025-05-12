// ---------------- [ File: ai-json-template-derive/src/expand_unnamed_variant_into_flat_justification.rs ]
crate::ix!();

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

    // 1) Gather expansions (top-level + each field)
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

    // 2) Build the final flattened variant snippet
    let flat_variant_ts = finalize_flat_unnamed_variant_ts(variant_ident, &expansions);

    // 3) Build the From arm
    let from_arm_ts = finalize_from_arm_unnamed_variant_ts(
        parent_enum_ident,
        variant_ident,
        justification_ident,
        confidence_ident,
        &expansions
    );

    (flat_variant_ts, from_arm_ts)
}

#[cfg(test)]
mod test_expand_unnamed_variant_into_flat_justification {
    use super::*;

    /// A simple “dummy” flattener for testing that just passes the same field names through.
    /// We do not produce any extra justification/conf in this stub.
    fn dummy_flatten_unnamed_field(
        field_ident: &Ident,
        _ty: &syn::Type,
        _skip_self: bool,
        _skip_child: bool
    ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
        let decl = quote! { #field_ident: #field_ident, };
        let init = quote! { #field_ident };
        // No justification/conf expansions in this dummy function
        (vec![decl], init, TokenStream2::new(), TokenStream2::new())
    }

    fn dummy_skip_field(_f: &syn::Field) -> bool { false }
    fn dummy_is_leaf_type(_t: &syn::Type) -> bool { false }

    #[traced_test]
    fn test_two_tuple_fields() {
        // We'll build an unnamed variant: (bool, String)
        let f0 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { bool },
        };
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { String },
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

        let parent = Ident::new("SomeEnum", proc_macro2::Span::call_site());
        let var = Ident::new("TupleVar", proc_macro2::Span::call_site());
        let just_id = Ident::new("SomeEnumJustification", proc_macro2::Span::call_site());
        let conf_id = Ident::new("SomeEnumConfidence", proc_macro2::Span::call_site());

        let (flat_ts, arm_ts) = expand_unnamed_variant_into_flat_justification(
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

        let flat_str = flat_ts.to_string();
        let from_str = arm_ts.to_string();

        // Basic checks for snippet presence:
        assert!(
            flat_str.contains("TupleVar {") && flat_str.contains("f0 : f0") && flat_str.contains("f1 : f1"),
            "Flat variant snippet should declare 'TupleVar' with f0, f1"
        );
        assert!(
            from_str.contains("FlatJustifiedSomeEnum :: TupleVar {")
            && from_str.contains("f0 , f1")
            && from_str.contains("SomeEnum :: TupleVar ( f0 , f1 )"),
            "From-arm snippet should pattern-match f0, f1 and construct the original enum"
        );
    }

    #[traced_test]
    fn test_expand_unnamed_variant_no_special_hacks() {
        // This is just a second scenario with a different enum/variant naming
        let f0 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { bool },
        };
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { String },
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

        let parent = syn::Ident::new("MyEnum", proc_macro2::Span::call_site());
        let var = syn::Ident::new("SomeTupleVariant", proc_macro2::Span::call_site());
        let just = syn::Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf = syn::Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());

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
            dummy_is_leaf_type
        );

        let fv_s = fv.to_string();
        let arm_s = arm.to_string();
        // Just quick checks:
        assert!(fv_s.contains("SomeTupleVariant {"));
        assert!(arm_s.contains("FlatJustifiedMyEnum :: SomeTupleVariant"));
    }
}
