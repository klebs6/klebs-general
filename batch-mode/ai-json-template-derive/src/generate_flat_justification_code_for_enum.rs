// ---------------- [ File: ai-json-template-derive/src/generate_flat_justification_code_for_enum.rs ]
crate::ix!();

pub fn generate_flat_justification_code_for_enum(
    enum_ident:                 &Ident,
    data_enum:                  &DataEnum,
    span:                       Span,
    skip_variant_self_just_fn:  impl Fn(&syn::Variant) -> bool,
    skip_variant_child_just_fn: impl Fn(&syn::Variant) -> bool,
    skip_field_self_just_fn:    impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:            impl Fn(&syn::Type) -> bool,

    flatten_named_field_fn:     impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),

    flatten_unnamed_field_fn:   impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
) -> (TokenStream2, TokenStream2)
{
    trace!("Starting generate_flat_justification_code_for_enum for '{}'", enum_ident);

    let (flat_enum_ident, justified_ident, justification_ident, confidence_ident) =
        create_flat_justification_idents_for_enum(enum_ident, span);

    let mut variant_defs = Vec::new();
    let mut from_arms    = Vec::new();

    for variant in &data_enum.variants {
        debug!("Processing variant '{}'", variant.ident);
        let (fv, arm) = generate_flat_variant_for_variant(
            enum_ident,
            variant,
            &justification_ident,
            &confidence_ident,
            &skip_variant_self_just_fn,
            &skip_variant_child_just_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
            &flatten_named_field_fn,
            &flatten_unnamed_field_fn,
        );
        variant_defs.push(fv);
        from_arms.push(arm);
    }

    let flat_ts = quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum #flat_enum_ident {
            #(#variant_defs)*
        }
    };

    let from_impl_ts = quote! {
        impl ::core::convert::From<#flat_enum_ident> for #justified_ident {
            fn from(flat: #flat_enum_ident) -> Self {
                match flat {
                    #(#from_arms),*
                }
            }
        }
    };

    debug!(
        "Completed generate_flat_justification_code_for_enum for '{}'",
        enum_ident
    );
    (flat_ts, from_impl_ts)
}

#[cfg(test)]
mod test_generate_flat_justification_code_for_enum {
    use super::*;

    // We'll define small stubs for "skip_variant_self_just_fn", etc.

    #[traced_test]
    fn test_full_expansion_unit_variant_only() {
        // Minimal enum with 1 unit variant
        let variant = Variant {
            attrs: vec![],
            ident: Ident::new("UnitVar", proc_macro2::Span::call_site()),
            fields: Fields::Unit,
            discriminant: None,
        };
        let data_enum = DataEnum {
            enum_token: Default::default(),
            brace_token: Default::default(),
            variants: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(variant);
                p
            },
        };

        let skip_variant_self_just = |_v: &syn::Variant| false;
        let skip_variant_child_just = |_v: &syn::Variant| false;
        let skip_field_self_just = |_f: &syn::Field| false;
        let is_leaf_type = |_t: &syn::Type| false;

        fn flatten_named_field_fn(
            _fid: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            (vec![TokenStream2::new()], TokenStream2::new(), TokenStream2::new(), TokenStream2::new())
        }
        fn flatten_unnamed_field_fn(
            _fid: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            (vec![TokenStream2::new()], TokenStream2::new(), TokenStream2::new(), TokenStream2::new())
        }

        let enum_ident = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = generate_flat_justification_code_for_enum(
            &enum_ident,
            &data_enum,
            proc_macro2::Span::call_site(),
            skip_variant_self_just,
            skip_variant_child_just,
            skip_field_self_just,
            is_leaf_type,
            flatten_named_field_fn,
            flatten_unnamed_field_fn
        );

        let flat_str = flat_ts.to_string();
        assert!(flat_str.contains("pub enum FlatJustifiedMyEnum"));
        assert!(flat_str.contains("UnitVar"));

        let from_str = from_ts.to_string();
        assert!(from_str.contains("From < FlatJustifiedMyEnum > for JustifiedMyEnum"));
        assert!(from_str.contains("FlatJustifiedMyEnum :: UnitVar =>"));
    }

    #[traced_test]
    fn test_full_expansion_with_named_and_unnamed() {
        // Build an enum with 2 variants:
        //   NamedVar { x: bool }
        //   TupleVar(u32)
        let named_var = Variant {
            attrs: vec![],
            ident: Ident::new("NamedVar", proc_macro2::Span::call_site()),
            fields: {
                let f = Field {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    ident: Some(Ident::new("x", proc_macro2::Span::call_site())),
                    colon_token: Some(Default::default()),
                    ty: syn::parse_quote! { bool },
                    mutability: FieldMutability::None,
                };
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f);
                Fields::Named(FieldsNamed {
                    brace_token: Default::default(),
                    named: p,
                })
            },
            discriminant: None,
        };
        let tuple_var = Variant {
            attrs: vec![],
            ident: Ident::new("TupleVar", proc_macro2::Span::call_site()),
            fields: {
                let f = Field {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    ident: None,
                    colon_token: None,
                    ty: syn::parse_quote! { u32 },
                    mutability: FieldMutability::None,
                };
                let mut p = syn::punctuated::Punctuated::new();
                p.push(f);
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: Default::default(),
                    unnamed: p,
                })
            },
            discriminant: None,
        };
        let data_enum = DataEnum {
            enum_token: Default::default(),
            brace_token: Default::default(),
            variants: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(named_var);
                p.push(tuple_var);
                p
            },
        };

        let skip_variant_self_just = |_v: &syn::Variant| false;
        let skip_variant_child_just = |_v: &syn::Variant| false;
        let skip_field_self_just = |_f: &syn::Field| false;
        let is_leaf_type = |_t: &syn::Type| false;

        fn dummy_flatten_named_field(
            field_ident: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            let decl = quote! { #field_ident: #field_ident, };
            let i_init = quote! { #field_ident };
            (vec![quote!{ #decl }], i_init, TokenStream2::new(), TokenStream2::new())
        }
        fn dummy_flatten_unnamed_field(
            field_ident: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2) {
            let decl = quote! { #field_ident: #field_ident, };
            let i_init = quote! { #field_ident };
            (vec![quote!{ #decl }], i_init, TokenStream2::new(), TokenStream2::new())
        }

        let enum_ident = Ident::new("MyEnum2", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = generate_flat_justification_code_for_enum(
            &enum_ident,
            &data_enum,
            proc_macro2::Span::call_site(),
            skip_variant_self_just,
            skip_variant_child_just,
            skip_field_self_just,
            is_leaf_type,
            dummy_flatten_named_field,
            dummy_flatten_unnamed_field
        );

        let fs = flat_ts.to_string();
        assert!(fs.contains("NamedVar {"));
        assert!(fs.contains("TupleVar {"));
        let fi = from_ts.to_string();
        assert!(fi.contains("FlatJustifiedMyEnum2 :: NamedVar"));
        assert!(fi.contains("FlatJustifiedMyEnum2 :: TupleVar"));
    }
}
