crate::ix!();

/// Expands an **unnamed (tuple) variant** (e.g. `MixedVariant(T, U)`) into “flat justification” form.
/// If not skipping self justification, we add top-level `enum_variant_justification`, `enum_variant_confidence`.
/// Then we flatten each tuple field, collecting them into `pat_vars` and final expansions.
pub fn expand_unnamed_variant_into_flat_justification(
    parent_enum_ident: &Ident,
    variant_ident: &Ident,
    unnamed_fields: &FieldsUnnamed,
    justification_ident: &Ident,
    confidence_ident: &Ident,
    skip_self_just: bool,
    skip_child_just: bool,
    flatten_unnamed_field_fn: impl Fn(
        &Ident,
        &syn::Type,
        bool,  // skip self
        bool   // skip child
    ) -> (TokenStream2, TokenStream2, TokenStream2, TokenStream2),

    is_justification_disabled_for_field_fn: impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn: impl Fn(&syn::Type) -> bool
) -> (TokenStream2, TokenStream2) {
    trace!(
        "Expanding unnamed variant '{}' in enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    let mut field_declarations = Vec::new();
    let mut pattern_vars       = Vec::new();
    let mut item_exprs         = Vec::new();
    let mut just_vals          = Vec::new();
    let mut conf_vals          = Vec::new();

    // Possibly add top-level justification & confidence
    if !skip_self_just {
        debug!(
            "Inserting top-level justification/conf for unnamed variant: {}",
            variant_ident
        );
        field_declarations.push(quote! {
            #[serde(default)]
            enum_variant_justification: String,
        });
        field_declarations.push(quote! {
            #[serde(default)]
            enum_variant_confidence: f32,
        });
        pattern_vars.push(quote! { enum_variant_justification });
        pattern_vars.push(quote! { enum_variant_confidence });
        just_vals.push(quote! { variant_justification: enum_variant_justification });
        conf_vals.push(quote! { variant_confidence: enum_variant_confidence });
    }

    for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
        let field_ident = Ident::new(&format!("f{}", idx), field.span());
        let skip_f_self = is_justification_disabled_for_field_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (decls, i_init, j_init, c_init) 
            = flatten_unnamed_field_fn(&field_ident, &field.ty, skip_f_self, child_skip);

        field_declarations.extend(decls);
        pattern_vars.push(quote! { #field_ident });

        if !i_init.is_empty() {
            item_exprs.push(i_init);
        }
        if !j_init.is_empty() {
            just_vals.push(j_init);
        }
        if !c_init.is_empty() {
            conf_vals.push(c_init);
        }
    }

    let flat_variant = quote! {
        #variant_ident {
            #(#field_declarations)*
        },
    };
    let item_constructor = quote! {
        #parent_enum_ident::#variant_ident( #( #item_exprs ),* )
    };
    let just_constructor = quote! {
        #justification_ident::#variant_ident {
            #( #just_vals ),*
        }
    };
    let conf_constructor = quote! {
        #confidence_ident::#variant_ident {
            #( #conf_vals ),*
        }
    };

    let from_arm = quote! {
        FlatJustified#parent_enum_ident::#variant_ident { #( #pattern_vars ),* } => {
            Self {
                item: #item_constructor,
                justification: #just_constructor,
                confidence:    #conf_constructor,
            }
        }
    };

    (flat_variant, from_arm)
}

#[cfg(test)]
mod test_expand_unnamed_variant_into_flat_justification {
    use super::*;
    use syn::{Field, FieldsUnnamed, Visibility};
    use traced_test::traced_test;

    #[traced_test]
    fn test_two_tuple_fields() {
        // We'll build an unnamed variant: (bool, String)
        let f0 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::parse_quote! { bool },
        };
        let f1 = Field {
            attrs: vec![],
            vis: Visibility::Inherited,
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

        fn dummy_flatten_unnamed_field(
            field_ident: &Ident,
            _ty: &syn::Type,
            _skip_self: bool,
            _skip_child: bool
        ) -> (TokenStream2, TokenStream2, TokenStream2, TokenStream2) {
            // Pretend we produce a single field in the flattened variant with the same name
            let decl = quote! { #field_ident: #field_ident, };
            (quote!{ #decl }, quote!{ #field_ident }, TokenStream2::new(), TokenStream2::new())
        }
        fn dummy_skip_field(_f: &syn::Field) -> bool { false }
        fn dummy_is_leaf_type(_t: &syn::Type) -> bool { false }

        let parent = Ident::new("SomeEnum", Span::call_site());
        let var = Ident::new("TupleVar", Span::call_site());
        let just_id = Ident::new("SomeEnumJustification", Span::call_site());
        let conf_id = Ident::new("SomeEnumConfidence", Span::call_site());

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
}
