// ---------------- [ File: ai-json-template-derive/src/expand_named_variant_into_flat_justification.rs ]
crate::ix!();

/// Expands a **named (struct) variant** (e.g. `StructVariant { x: T, y: U }`)
/// into “flat justification” form. We generate additional top-level justification/conf
/// if `skip_self_just` is `false`. For each field, we flatten it as well.
pub fn expand_named_variant_into_flat_justification(
    parent_enum_ident: &Ident,
    variant_ident: &Ident,
    named_fields: &FieldsNamed,
    justification_ident: &Ident,
    confidence_ident: &Ident,
    skip_self_just: bool,
    skip_child_just: bool,
    // These two callbacks represent your existing “flatten_named_field” or “is_justification_disabled_for_field”, etc.
    flatten_named_field_fn: impl Fn(
        &Ident,        // field name
        &syn::Type, 
        bool,          // skip self just
        bool           // skip child just
    ) -> (TokenStream2, TokenStream2, TokenStream2, TokenStream2),

    is_justification_disabled_for_field_fn: impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn: impl Fn(&syn::Type) -> bool
) -> (TokenStream2, TokenStream2) {
    trace!(
        "Expanding named variant '{}' of enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    let mut field_declarations = Vec::new();
    let mut pattern_vars       = Vec::new();
    let mut item_inits         = Vec::new();
    let mut just_inits         = Vec::new();
    let mut conf_inits         = Vec::new();

    // Possibly add top-level variant_justification & variant_confidence
    if !skip_self_just {
        debug!("Inserting top-level enum_variant_just/conf for variant: {}", variant_ident);
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

        just_inits.push(quote! { variant_justification: enum_variant_justification });
        conf_inits.push(quote! { variant_confidence: enum_variant_confidence });
    }

    // Flatten each named field
    for field in &named_fields.named {
        let f_ident = match field.ident {
            Some(ref id) => id,
            None => {
                warn!("Ignoring unnamed field in 'named variant' expansion, which is unusual!");
                continue;
            }
        };

        let skip_f_self = is_justification_disabled_for_field_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (field_decls, i_init, j_init, c_init) 
            = flatten_named_field_fn(f_ident, &field.ty, skip_f_self, child_skip);

        field_declarations.extend(field_decls);

        // We'll capture this field in the pattern: e.g. `text`
        pattern_vars.push(quote! { #f_ident });

        if !i_init.is_empty() {
            item_inits.push(i_init);
        }
        if !j_init.is_empty() {
            just_inits.push(j_init);
        }
        if !c_init.is_empty() {
            conf_inits.push(c_init);
        }
    }

    let flat_variant = quote! {
        #variant_ident {
            #(#field_declarations)*
        },
    };

    // Build final arms
    let field_idents: Vec<_> = named_fields.named
        .iter()
        .filter_map(|f| f.ident.clone())
        .collect();

    let item_constructor = quote! {
        #parent_enum_ident::#variant_ident {
            #( #field_idents: #item_inits ),*
        }
    };
    let just_constructor = quote! {
        #justification_ident::#variant_ident {
            #( #just_inits ),*
        }
    };
    let conf_constructor = quote! {
        #confidence_ident::#variant_ident {
            #( #conf_inits ),*
        }
    };

    let from_arm = quote! {
        FlatJustified #parent_enum_ident::#variant_ident { #( #pattern_vars ),* } => {
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
        ) -> (TokenStream2, TokenStream2, TokenStream2, TokenStream2) {
            // For testing, pretend we produce one flattened field with same name, 
            // plus no justification/conf expansions. 
            // That’s enough to test this subroutine’s logic of pattern matching.
            let decl = quote! { #field_ident: #field_ident, };
            let i_init = quote! { #field_ident };
            (quote!{ #decl }, i_init, TokenStream2::new(), TokenStream2::new())
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
