// ---------------- [ File: ai-json-template-derive/src/generate_flat_variant_for_variant.rs ]
crate::ix!();

pub fn generate_flat_variant_for_variant(
    enum_ident:                 &Ident,
    variant:                    &syn::Variant,
    justification_ident:        &Ident,
    confidence_ident:           &Ident,
    skip_variant_self_just_fn:  &impl Fn(&syn::Variant) -> bool,
    skip_variant_child_just_fn: &impl Fn(&syn::Variant) -> bool,
    skip_field_self_just_fn:    &impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:            &impl Fn(&syn::Type) -> bool,
    flatten_named_field_fn:     &impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
    flatten_unnamed_field_fn:   &impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
) -> (TokenStream2, TokenStream2) {
    trace!(
        "generate_flat_variant_for_variant called for '{}::{}'",
        enum_ident,
        variant.ident
    );

    let skip_self_just  = skip_variant_self_just_fn(variant);
    let skip_child_just = skip_self_just || skip_variant_child_just_fn(variant);

    match &variant.fields {
        Fields::Unit => {
            expand_unit_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                justification_ident,
                confidence_ident,
                skip_self_just
            )
        }
        Fields::Named(named_fields) => {
            expand_named_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                named_fields,
                justification_ident,
                confidence_ident,
                skip_self_just,
                skip_child_just,
                flatten_named_field_fn,
                skip_field_self_just_fn,
                is_leaf_type_fn
            )
        }
        Fields::Unnamed(unnamed_fields) => {
            expand_unnamed_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                unnamed_fields,
                justification_ident,
                confidence_ident,
                skip_self_just,
                skip_child_just,
                flatten_unnamed_field_fn,
                skip_field_self_just_fn,
                is_leaf_type_fn
            )
        }
    }
}

#[cfg(test)]
mod verify_generate_flat_variant_for_variant {
    use super::*;

    fn mock_skip_variant_self_just_fn(v: &Variant) -> bool {
        let result = is_justification_disabled_for_variant(v);
        debug!("mock_skip_variant_self_just_fn => Variant '{}', returning {}", v.ident, result);
        result
    }

    fn mock_skip_variant_child_just_fn(v: &Variant) -> bool {
        let result = is_justification_disabled_for_inner_variant(v);
        debug!("mock_skip_variant_child_just_fn => Variant '{}', returning {}", v.ident, result);
        result
    }

    fn mock_skip_field_self_just_fn(f: &Field) -> bool {
        let result = is_justification_disabled_for_field(f);
        debug!("mock_skip_field_self_just_fn => Field '{:?}', returning {}", f.ident, result);
        result
    }

    fn mock_is_leaf_type_fn(ty: &syn::Type) -> bool {
        let result = is_leaf_type(ty);
        debug!("mock_is_leaf_type_fn => Type '{}', returning {}", quote!(#ty), result);
        result
    }

    #[traced_test]
    fn ensure_unit_variant_skip_self_false() {
        trace!("Starting test: ensure_unit_variant_skip_self_false");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        // Create a unit variant
        let variant: Variant = parse_quote! {
            #[some_meta]
            Unit
        };

        // We'll forcibly override any attributes so skip_self_just becomes false.
        // The normal detection might rely on an actual attribute, but let's ensure we test logic here.
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| false, // skip_variant_self_just_fn => false
            &|_v| false, // skip_variant_child_just_fn => false
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        info!("Resulting 'flat_ts' = {}", quote!(#flat_ts));
        info!("Resulting 'from_ts' = {}", quote!(#from_ts));

        // Check for mention of "variant_justification" or "variant_confidence"
        let flat_ts_str = quote!(#flat_ts).to_string();
        assert!(
            flat_ts_str.contains("variant_justification") && flat_ts_str.contains("variant_confidence"),
            "Expected top-level variant_justification/confidence fields in the flattened definition"
        );

        // Check from_ts for correct pattern usage
        let from_ts_str = quote!(#from_ts).to_string();
        assert!(
            from_ts_str.contains("Self {")
            && from_ts_str.contains("variant_justification")
            && from_ts_str.contains("variant_confidence"),
            "Expected correct from-arm expansions in 'from_ts'"
        );
    }

    #[traced_test]
    fn ensure_unit_variant_skip_self_true() {
        trace!("Starting test: ensure_unit_variant_skip_self_true");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        // Create a unit variant
        let variant: Variant = parse_quote! {
            Unit
        };

        // We'll forcibly override any attributes so skip_self_just becomes true:
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| true,  // skip_variant_self_just_fn => true
            &|_v| false, // skip_variant_child_just_fn => false
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        debug!("Resulting 'flat_ts' = {}", quote!(#flat_ts));
        debug!("Resulting 'from_ts' = {}", quote!(#from_ts));

        // For skip_self_just = true, we expect NO top-level justification/conf.
        let flat_ts_str = quote!(#flat_ts).to_string();
        assert!(
            !flat_ts_str.contains("variant_justification")
            && !flat_ts_str.contains("variant_confidence"),
            "Should NOT contain top-level variant_justification/confidence"
        );

        let from_ts_str = quote!(#from_ts).to_string();
        assert!(
            from_ts_str.contains("Self {")
            && !from_ts_str.contains("variant_justification")
            && !from_ts_str.contains("variant_confidence"),
            "Should NOT contain top-level variant_justification/confidence in from_ts"
        );
    }

    #[traced_test]
    fn ensure_named_variant_skip_self_false_child_false() {
        trace!("Starting test: ensure_named_variant_skip_self_false_child_false");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        // A named variant with some fields
        let variant: Variant = parse_quote! {
            DetailedVariant {
                alpha: String,
                #[justify=false]
                beta: i32,
            }
        };

        // skip_self_just = false, skip_child_just = false => top-level + child fields
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| false,
            &|_v| false,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        debug!("flat_ts => {}", quote!(#flat_ts));
        debug!("from_ts => {}", quote!(#from_ts));

        let flat_str = quote!(#flat_ts).to_string();

        // Expect variant_justification, variant_confidence
        assert!(
            flat_str.contains("variant_justification")
            && flat_str.contains("variant_confidence"),
            "Should contain top-level variant justification/confidence"
        );

        // Expect alpha and alpha_justification, alpha_confidence
        assert!(
            flat_str.contains("alpha")
            && flat_str.contains("alpha_justification")
            && flat_str.contains("alpha_confidence"),
            "Named field alpha + alpha_just/conf not found"
        );

        // For beta, justification is set to false via attribute => no beta_just/conf
        assert!(
            flat_str.contains("beta")
            && !flat_str.contains("beta_justification")
            && !flat_str.contains("beta_confidence"),
            "Should not contain beta_justification/conf"
        );

        // Check from_ts pattern arms
        let from_str = quote!(#from_ts).to_string();
        assert!(
            from_str.contains("alpha: ::core::convert::From::from(alpha)")
            && from_str.contains("variant_justification")
            && from_str.contains("variant_confidence"),
            "Expect from_ts to convert alpha field + top-level justification/conf"
        );
    }

    #[traced_test]
    fn ensure_named_variant_skip_self_true_child_false() {
        trace!("Starting test: ensure_named_variant_skip_self_true_child_false");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        let variant: Variant = parse_quote! {
            AnotherVariant {
                name: String,
                count: u32
            }
        };

        // skip_self_just = true => No top-level justification/conf
        // skip_child_just = false => child fields get expansions
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| true,
            &|_v| false,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        let flat_str = quote!(#flat_ts).to_string();
        assert!(
            !flat_str.contains("variant_justification")
            && !flat_str.contains("variant_confidence"),
            "skip_self_just => no top-level justification/conf"
        );

        // For both fields => name + name_just/conf, count + count_just/conf
        assert!(
            flat_str.contains("name")
            && flat_str.contains("name_justification")
            && flat_str.contains("name_confidence")
            && flat_str.contains("count")
            && flat_str.contains("count_justification")
            && flat_str.contains("count_confidence"),
            "Fields must have justification/conf placeholders"
        );

        let from_str = quote!(#from_ts).to_string();
        assert!(
            from_str.contains("name: ::core::convert::From::from(name)")
            && from_str.contains("count: ::core::convert::From::from(count)")
            && !from_str.contains("variant_justification"),
            "Expect from_ts to omit top-level justification/conf but handle child fields"
        );
    }

    #[traced_test]
    fn ensure_named_variant_skip_self_false_child_true() {
        trace!("Starting test: ensure_named_variant_skip_self_false_child_true");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        let variant: Variant = parse_quote! {
            NamedVar {
                config: CustomType,
                size: f64
            }
        };

        // skip_self_just = false => top-level present
        // skip_child_just = true => child fields are treated as leaf => no child expansions
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| false,
            &|_v| true,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        let flat_str = quote!(#flat_ts).to_string();
        // top-level
        assert!(
            flat_str.contains("variant_justification") && flat_str.contains("variant_confidence"),
            "top-level justification/conf must be present"
        );

        // child => config, size but no config_just/conf if skip_self_just is not triggered
        // Actually skip_child_just = true => we do NOT do `From::from(field)` for subfields, we treat them as is
        // but we DO still generate "config_justification" if skip_self_just is false for that field. 
        // We'll rely on the default skip logic for each field, but let's just check we see config + config_just/conf.
        assert!(
            flat_str.contains("config")
            && flat_str.contains("config_justification")
            && flat_str.contains("config_confidence"),
            "config field must have justification/conf placeholders"
        );

        // For size => same approach => it's presumably a numeric leaf
        assert!(
            flat_str.contains("size")
            && flat_str.contains("size_justification")
            && flat_str.contains("size_confidence"),
            "size field must have justification/conf placeholders"
        );

        // Check from_ts
        let from_str = quote!(#from_ts).to_string();
        assert!(
            from_str.contains("variant_confidence")
            && from_str.contains("variant_justification")
            && !from_str.contains("::core::convert::From::from(config)")
            && from_str.contains("config: config") // skip_child => no From::from
            && from_str.contains("size: size"),
            "We should see top-level justification/conf, but child fields not converted (skip_child_just=true)."
        );
    }

    #[traced_test]
    fn ensure_unnamed_variant_skip_self_false_child_false() {
        trace!("Starting test: ensure_unnamed_variant_skip_self_false_child_false");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        let variant: Variant = parse_quote! {
            TupleVar ( String, i32 )
        };

        // skip_self_just = false => top-level
        // skip_child_just = false => subfields get expansions
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| false,
            &|_v| false,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        let flat_str = quote!(#flat_ts).to_string();
        assert!(
            flat_str.contains("variant_justification")
            && flat_str.contains("variant_confidence"),
            "Should contain top-level justification/conf"
        );
        // field_0, field_1 + expansions
        assert!(
            flat_str.contains("f0")
            && flat_str.contains("f0_justification")
            && flat_str.contains("f0_confidence")
            && flat_str.contains("f1")
            && flat_str.contains("f1_justification")
            && flat_str.contains("f1_confidence"),
            "Unnamed fields expansions not found"
        );

        let from_str = quote!(#from_ts).to_string();
        assert!(
            from_str.contains("variant_justification")
            && from_str.contains("variant_confidence")
            && from_str.contains("::core::convert::From::from(f0)")
            && from_str.contains("::core::convert::From::from(f1)"),
            "Expected from-arm expansions with top-level + child conversions"
        );
    }

    #[traced_test]
    fn ensure_unnamed_variant_skip_self_true_child_false() {
        trace!("Starting test: ensure_unnamed_variant_skip_self_true_child_false");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        let variant: Variant = parse_quote! {
            AnotherTuple ( bool, CustomType )
        };

        // skip_self_just=true => no top-level
        // skip_child_just=false => child expansions
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| true,
            &|_v| false,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        let flat_str = quote!(#flat_ts).to_string();
        assert!(
            !flat_str.contains("variant_justification")
            && !flat_str.contains("variant_confidence"),
            "Top-level justification/conf should NOT appear"
        );

        // child expansions
        assert!(
            flat_str.contains("f0")
            && flat_str.contains("f0_justification")
            && flat_str.contains("f0_confidence")
            && flat_str.contains("f1")
            && flat_str.contains("f1_justification")
            && flat_str.contains("f1_confidence"),
            "Child expansions for unnamed fields must appear"
        );

        let from_str = quote!(#from_ts).to_string();
        assert!(
            from_str.contains("Self {")
            && !from_str.contains("variant_justification")
            && !from_str.contains("variant_confidence")
            && from_str.contains("::core::convert::From::from(f0)")
            && from_str.contains("::core::convert::From::from(f1)"),
            "No top-level justification/conf in from-arm, but child expansions must appear"
        );
    }

    #[traced_test]
    fn ensure_unnamed_variant_skip_self_false_child_true() {
        trace!("Starting test: ensure_unnamed_variant_skip_self_false_child_true");
        let enum_ident = parse_quote!(MyEnum);
        let justification_ident = parse_quote!(MyEnumJustification);
        let confidence_ident = parse_quote!(MyEnumConfidence);

        let variant: Variant = parse_quote! {
            CombinedTuple( i16, AnotherType )
        };

        // skip_self_just=false => top-level
        // skip_child_just=true => subfields are treated as leaf (no From::from calls)
        let (flat_ts, from_ts) = generate_flat_variant_for_variant(
            &enum_ident,
            &variant,
            &justification_ident,
            &confidence_ident,
            &|_v| false,
            &|_v| true,
            &mock_skip_field_self_just_fn,
            &mock_is_leaf_type_fn,
            &flatten_named_field,
            &flatten_unnamed_field
        );

        let flat_str = quote!(#flat_ts).to_string();
        // top-level
        assert!(
            flat_str.contains("variant_justification")
            && flat_str.contains("variant_confidence"),
            "Expect top-level justification/conf"
        );
        // child => f0, f1, plus f0_just/conf, f1_just/conf
        assert!(
            flat_str.contains("f0")
            && flat_str.contains("f1")
            && flat_str.contains("f0_justification")
            && flat_str.contains("f1_justification")
            && flat_str.contains("f0_confidence")
            && flat_str.contains("f1_confidence"),
            "Unnamed fields expansions not found"
        );

        let from_str = quote!(#from_ts).to_string();
        // We do not do From::from => skip_child_just => treat them as leaf
        assert!(
            from_str.contains("item: MyEnum :: CombinedTuple(f0, f1)")
            && from_str.contains("variant_justification")
            && from_str.contains("variant_confidence"),
            "Should see direct usage of f0, f1 + top-level justification/conf"
        );
    }
}
