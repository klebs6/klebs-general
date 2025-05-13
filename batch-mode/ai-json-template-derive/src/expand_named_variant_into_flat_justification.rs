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

    /// A helper to create a `syn::Ident` from a &str
    fn ident_of(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    /// A helper to create a mock named field, with its own `syn::Ident` and `syn::Type`.
    /// This function does not apply any attributes, as that's optional for these tests.
    fn mock_named_field(field_name: &str, type_str: &str) -> Field {
        // We'll parse the type string into a syn::Type for testing.
        let ty = Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: {
                    let mut segs = syn::punctuated::Punctuated::<PathSegment, PathSep>::new();
                    segs.push(PathSegment::from(Ident::new(type_str, Span::call_site())));
                    segs
                },
            },
        });

        Field {
            // We won't attach attributes here, can add if needed for skip tests
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            ident: Some(ident_of(field_name)),
            colon_token: Some(Default::default()),
            ty,
            mutability: FieldMutability::None,
        }
    }

    /// Mocks a closure for `skip_field_self_just_fn`.
    /// Returns `true` if the field name contains "skipSelfJust", else false.
    /// This is purely for testing the branching logic.
    fn mock_skip_field_self_just_fn(field: &Field) -> bool {
        if let Some(id) = &field.ident {
            let name = id.to_string();
            return name.contains("skipSelfJust");
        }
        false
    }

    /// Mocks a closure for `is_leaf_type_fn`.
    /// We'll treat any type named "u8" or "String" as "leaf".
    /// Otherwise, we treat it as non-leaf to check code flow.
    fn mock_is_leaf_type_fn(ty: &Type) -> bool {
        if let Type::Path(tp) = ty {
            if let Some(seg) = tp.path.segments.last() {
                let name = seg.ident.to_string();
                if name == "u8" || name == "String" {
                    return true;
                }
            }
        }
        false
    }

    /// Mocks a closure for `flatten_named_field_fn`.
    /// This is used by the function under test to flatten each named field into tokens.
    /// We'll produce mock expansions that log the field name and skip status.
    fn mock_flatten_named_field_fn(
        field_ident: &Ident,
        _field_ty: &Type,
        skip_self_just: bool,
        parent_skip_child: bool,
    ) -> (
        Vec<TokenStream2>,
        TokenStream2,
        TokenStream2,
        TokenStream2
    ) {
        trace!(
            "mock_flatten_named_field_fn called => field='{}', skip_self_just={}, parent_skip_child={}",
            field_ident, skip_self_just, parent_skip_child
        );

        // A simple representation for demonstration
        let decls = vec![quote::quote! { /* mock decl for #field_ident */ }];
        let item_init = quote::quote! { /* mock item_init for #field_ident */ };
        let just_init = if skip_self_just {
            quote::quote! { /* no justification for #field_ident */ }
        } else {
            quote::quote! { /* justification for #field_ident */ }
        };
        let conf_init = if skip_self_just {
            quote::quote! { /* no confidence for #field_ident */ }
        } else {
            quote::quote! { /* confidence for #field_ident */ }
        };

        (decls, item_init, just_init, conf_init)
    }

    /// Constructs a mock `FieldsNamed` from a list of `(field_name, type_str)` pairs.
    fn build_mock_fields_named(fields: &[(&str, &str)]) -> FieldsNamed {
        let named_list: Vec<Field> = fields
            .iter()
            .map(|(f_name, t_str)| mock_named_field(f_name, t_str))
            .collect();

        syn::FieldsNamed {
            brace_token: Default::default(),
            named: named_list.into_iter().collect(),
        }
    }

    /// Verifies that the returned TokenStreams are not empty (at a minimum).
    /// In real usage, we might parse them back or assert on more specifics.
    fn assert_valid_tokens(
        flat_variant_ts: &TokenStream2,
        from_arm_ts: &TokenStream2,
        scenario_desc: &str
    ) {
        debug!("Asserting the tokens for scenario '{}'", scenario_desc);
        let flat_str = flat_variant_ts.to_string();
        let from_str = from_arm_ts.to_string();

        assert!(!flat_str.is_empty(),
            "Expected non-empty flat_variant_ts for scenario '{}'.", scenario_desc
        );
        assert!(!from_str.is_empty(),
            "Expected non-empty from_arm_ts for scenario '{}'.", scenario_desc
        );
        debug!(
            "Scenario '{}': Passed => flat_variant_ts.len={}, from_arm_ts.len={}",
            scenario_desc,
            flat_str.len(),
            from_str.len()
        );
    }

    #[traced_test]
    fn test_named_variant_no_fields() {
        info!("Running test_named_variant_no_fields");
        let parent_enum_ident = ident_of("MyEnum");
        let variant_ident = ident_of("EmptyVariant");
        let named_fields = build_mock_fields_named(&[]); // no fields
        let justification_ident = ident_of("MyEnumJustification");
        let confidence_ident = ident_of("MyEnumConfidence");
        let skip_self_just = false;
        let skip_child_just = false;

        trace!("Invoking expand_named_variant_into_flat_justification with no fields");
        let (flat_variant_ts, from_arm_ts) = expand_named_variant_into_flat_justification(
            &parent_enum_ident,
            &variant_ident,
            &named_fields,
            &justification_ident,
            &confidence_ident,
            skip_self_just,
            skip_child_just,
            mock_flatten_named_field_fn,
            mock_skip_field_self_just_fn,
            mock_is_leaf_type_fn,
        );

        assert_valid_tokens(&flat_variant_ts, &from_arm_ts, "test_named_variant_no_fields");
    }

    #[traced_test]
    fn test_named_variant_single_field_no_skip() {
        info!("Running test_named_variant_single_field_no_skip");
        let parent_enum_ident = ident_of("SomeEnum");
        let variant_ident = ident_of("SingleFieldVariant");
        let named_fields = build_mock_fields_named(&[("alpha", "u8")]);
        let justification_ident = ident_of("SomeEnumJustification");
        let confidence_ident = ident_of("SomeEnumConfidence");
        let skip_self_just = false;
        let skip_child_just = false;

        trace!("Invoking expand_named_variant_into_flat_justification with a single field, no skip");
        let (flat_variant_ts, from_arm_ts) = expand_named_variant_into_flat_justification(
            &parent_enum_ident,
            &variant_ident,
            &named_fields,
            &justification_ident,
            &confidence_ident,
            skip_self_just,
            skip_child_just,
            mock_flatten_named_field_fn,
            mock_skip_field_self_just_fn,
            mock_is_leaf_type_fn,
        );

        assert_valid_tokens(&flat_variant_ts, &from_arm_ts, "test_named_variant_single_field_no_skip");
    }

    #[traced_test]
    fn test_named_variant_multiple_fields_skip_self_just() {
        info!("Running test_named_variant_multiple_fields_skip_self_just");
        let parent_enum_ident = ident_of("MultiEnum");
        let variant_ident = ident_of("MultiFieldVariant");
        // We'll have a few fields, some with "skipSelfJust" in the name
        let named_fields = build_mock_fields_named(&[
            ("alpha", "u8"),
            ("skipSelfJust_beta", "String"),
            ("gamma", "u8"),
        ]);
        let justification_ident = ident_of("MultiEnumJustification");
        let confidence_ident = ident_of("MultiEnumConfidence");
        let skip_self_just = true;
        let skip_child_just = false;

        trace!("Invoking with skip_self_just=true => top-level just/conf omitted, but field self-just remains for fields not containing 'skipSelfJust'");
        let (flat_variant_ts, from_arm_ts) = expand_named_variant_into_flat_justification(
            &parent_enum_ident,
            &variant_ident,
            &named_fields,
            &justification_ident,
            &confidence_ident,
            skip_self_just,
            skip_child_just,
            mock_flatten_named_field_fn,
            mock_skip_field_self_just_fn,
            mock_is_leaf_type_fn,
        );

        assert_valid_tokens(
            &flat_variant_ts,
            &from_arm_ts,
            "test_named_variant_multiple_fields_skip_self_just"
        );
    }

    #[traced_test]
    fn test_named_variant_skip_child_just() {
        info!("Running test_named_variant_skip_child_just");
        let parent_enum_ident = ident_of("ChildEnum");
        let variant_ident = ident_of("ChildVariant");
        // We'll define fields that would normally need flattening
        let named_fields = build_mock_fields_named(&[
            ("field1", "ComplexTypeA"),
            ("field2", "u8"),
        ]);
        let justification_ident = ident_of("ChildEnumJustification");
        let confidence_ident = ident_of("ChildEnumConfidence");
        let skip_self_just = false;
        let skip_child_just = true;

        trace!("Invoking expand_named_variant_into_flat_justification with skip_child_just=true => child flattening is minimal");
        let (flat_variant_ts, from_arm_ts) = expand_named_variant_into_flat_justification(
            &parent_enum_ident,
            &variant_ident,
            &named_fields,
            &justification_ident,
            &confidence_ident,
            skip_self_just,
            skip_child_just,
            mock_flatten_named_field_fn,
            mock_skip_field_self_just_fn,
            mock_is_leaf_type_fn,
        );

        assert_valid_tokens(
            &flat_variant_ts,
            &from_arm_ts,
            "test_named_variant_skip_child_just"
        );
    }
}
