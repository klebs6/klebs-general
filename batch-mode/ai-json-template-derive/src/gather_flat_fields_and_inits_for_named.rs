// ---------------- [ File: ai-json-template-derive/src/gather_flat_fields_and_inits_for_named.rs ]
crate::ix!();

pub fn gather_flat_fields_and_inits_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    flat_fields: &mut Vec<proc_macro2::TokenStream>,
    item_inits: &mut Vec<proc_macro2::TokenStream>,
    just_inits: &mut Vec<proc_macro2::TokenStream>,
    conf_inits: &mut Vec<proc_macro2::TokenStream>,
) {
    trace!("gather_flat_fields_and_inits_for_named: starting for '{}'", ty_ident);

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Encountered unnamed field in a named struct; skipping");
                continue;
            }
        };

        let skip_self_just = is_justification_disabled_for_field(field);
        let skip_child_just =
            skip_self_just || is_justification_disabled_for_inner(field) || is_leaf_type(&field.ty);

        debug!(
            "Field '{}' => skip_self_just={} skip_child_just={}",
            field_ident, skip_self_just, skip_child_just
        );

        match compute_flat_type_for_stamped(&field.ty, skip_child_just, field.span()) {
            Ok(flattened_type) => {
                flat_fields.push(quote! {
                    #[serde(default)]
                    pub #field_ident: #flattened_type,
                });
            }
            Err(e) => {
                error!("Error flattening field '{}': {:?}", field_ident, e);
                flat_fields.push(e.to_compile_error());
                continue;
            }
        }

        // item_inits => direct or From::from
        if skip_child_just {
            item_inits.push(quote! {
                #field_ident: flat.#field_ident
            });
        } else {
            item_inits.push(quote! {
                #field_ident: ::core::convert::From::from(flat.#field_ident)
            });
        }

        // top-level justification/conf if not skip_self_just
        if !skip_self_just {
            let j_id = syn::Ident::new(
                &format!("{}_justification", field_ident),
                field_ident.span()
            );
            let c_id = syn::Ident::new(
                &format!("{}_confidence", field_ident),
                field_ident.span()
            );

            flat_fields.push(quote! {
                #[serde(default)]
                pub #j_id: String,
                #[serde(default)]
                pub #c_id: f32,
            });

            if skip_child_just {
                just_inits.push(quote! { #j_id: flat.#j_id });
                conf_inits.push(quote! { #c_id: flat.#c_id });
            } else {
                let child_just_ty = child_ty_to_just(&field.ty);
                let child_conf_ty = child_ty_to_conf(&field.ty);

                just_inits.push(quote! {
                    #j_id: #child_just_ty {
                        detail_justification: flat.#j_id,
                        ..::core::default::Default::default()
                    }
                });
                conf_inits.push(quote! {
                    #c_id: #child_conf_ty {
                        detail_confidence: flat.#c_id,
                        ..::core::default::Default::default()
                    }
                });
            }
        }
    }

    trace!("gather_flat_fields_and_inits_for_named: done collecting fields and inits.");
}

#[cfg(test)]
mod test_gather_flat_fields_and_inits_for_named {
    use super::*;

    #[traced_test]
    fn test_empty_named_struct() {
        trace!("Starting test_empty_named_struct");
        // Prepare an empty named struct:
        let item_struct: ItemStruct = parse_quote! {
            struct EmptyStruct {}
        };

        let fields = match item_struct.fields {
            Fields::Named(ref named) => named,
            _ => panic!("Expected named fields"),
        };

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        // Expect no expansions
        assert_eq!(flat_fields.len(), 0, "No fields should have been generated");
        assert_eq!(item_inits.len(), 0,  "No item inits expected for empty struct");
        assert_eq!(just_inits.len(), 0,  "No justification inits for empty struct");
        assert_eq!(conf_inits.len(), 0,  "No confidence inits for empty struct");
        debug!("test_empty_named_struct passed");
    }

    #[traced_test]
    fn test_single_field_no_justify() {
        trace!("Starting test_single_field_no_justify");
        // This field has no special attributes => justification is not turned off => skip_self_just=false
        let item_struct: ItemStruct = parse_quote! {
            struct SingleField {
                my_number: i32
            }
        };

        let fields = match item_struct.fields {
            Fields::Named(ref named) => named,
            _ => panic!("Expected named fields"),
        };

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        // We expect exactly 1 normal flattened field plus 2 justification fields (my_number_justification, my_number_confidence).
        let flat_string = flat_fields.iter().map(|ts| ts.to_string()).collect::<Vec<_>>().join("\n");
        trace!("Flat fields:\n{}", flat_string);

        // 1) The flattened field
        assert!(
            flat_string.contains("my_number: i32"),
            "Should contain flattened field for `my_number`"
        );
        // 2) The justification field
        assert!(
            flat_string.contains("my_number_justification: String"),
            "Should contain my_number_justification"
        );
        // 3) The confidence field
        assert!(
            flat_string.contains("my_number_confidence: f32"),
            "Should contain my_number_confidence"
        );

        // item_inits => calls From::from(...) if skip_child_just == false
        // i32 is a leaf, but there's no #[justify_inner=false], so skip_child_just is true
        // because is_leaf_type(i32) => skip_child_just ends up being true => direct assignment
        assert_eq!(item_inits.len(), 1, "Should have 1 item init");
        let init_str = item_inits[0].to_string();
        debug!("item_inits[0] => {}", init_str);
        assert!(
            init_str.contains("my_number: flat . my_number"),
            "Expected direct assignment for i32 leaf"
        );

        // justification / confidence inits => because skip_self_just=false but skip_child_just=true => direct assignment
        assert_eq!(just_inits.len(), 1, "Should have 1 just init");
        assert_eq!(conf_inits.len(), 1, "Should have 1 conf init");
        let just_init_str = just_inits[0].to_string();
        let conf_init_str = conf_inits[0].to_string();
        debug!("just_inits[0] => {}", just_init_str);
        debug!("conf_inits[0] => {}", conf_init_str);

        assert!(
            just_init_str.contains("my_number_justification : flat . my_number_justification"),
            "Should be direct assignment for justification of leaf field"
        );
        assert!(
            conf_init_str.contains("my_number_confidence : flat . my_number_confidence"),
            "Should be direct assignment for confidence of leaf field"
        );

        debug!("test_single_field_no_justify passed");
    }

    #[traced_test]
    fn test_field_with_justify_false() {
        trace!("Starting test_field_with_justify_false");
        // This field has #[justify = false] => skip_self_just=true
        // The type is i32 => is_leaf_type => skip_child_just also will be true
        let item_struct: ItemStruct = parse_quote! {
            struct SkipJustifyField {
                #[justify=false]
                hidden: i32
            }
        };

        let fields = match item_struct.fields {
            Fields::Named(ref named) => named,
            _ => panic!("Expected named fields"),
        };

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        // We expect exactly 1 normal flattened field, no justification/conf fields because skip_self_just
        let flat_string = flat_fields.iter().map(|ts| ts.to_string()).collect::<Vec<_>>().join("\n");
        trace!("Flat fields:\n{}", flat_string);

        // 1) The flattened field
        assert!(
            flat_string.contains("hidden: i32"),
            "Should contain flattened field for `hidden`"
        );
        // 2) No hidden_justification
        assert!(
            !flat_string.contains("hidden_justification"),
            "Should NOT contain hidden_justification"
        );
        // 3) No hidden_confidence
        assert!(
            !flat_string.contains("hidden_confidence"),
            "Should NOT contain hidden_confidence"
        );

        // item_inits => direct assignment
        assert_eq!(item_inits.len(), 1);
        let init_str = item_inits[0].to_string();
        debug!("item_inits => {}", init_str);
        assert!(
            init_str.contains("hidden: flat . hidden"),
            "Should contain direct assignment"
        );

        // justification / confidence => none
        assert!(just_inits.is_empty(), "No justification init for justify=false");
        assert!(conf_inits.is_empty(), "No confidence init for justify=false");

        debug!("test_field_with_justify_false passed");
    }

    #[traced_test]
    fn test_complex_field_nonleaf() {
        trace!("Starting test_complex_field_nonleaf");
        // Suppose we have a custom type. 
        // We'll define a placeholder struct here to ensure it's recognized as non-leaf => 
        // is_leaf_type returns false => skip_child_just = false unless there's a direct attribute. 
        // We'll rely on the existing logic to handle child -> from(...) usage.

        let item_struct: ItemStruct = parse_quote! {
            struct ComplexHolder {
                // no explicit #[justify] => skip_self_just=false
                sub_thing: SomeCustomType
            }
        };

        // We'll also define a mock "SomeCustomType" so parse doesn't fail:
        // (In real usage, "SomeCustomType" might be declared elsewhere, but we only need it syntactically).
        // This is just to ensure parse_quote won't fail. We'll never do anything with it at runtime.
        #[allow(dead_code)]
        struct SomeCustomType;

        let fields = match item_struct.fields {
            Fields::Named(ref named) => named,
            _ => panic!("Expected named fields"),
        };

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        let flat_string = flat_fields.iter().map(|ts| ts.to_string()).collect::<Vec<_>>().join("\n");
        trace!("Flat fields:\n{}", flat_string);

        // We expect sub_thing => something like "FlatJustifiedSomeCustomType"
        // (since skip_child_just is false => we attempt to flatten child).
        assert!(
            flat_string.contains("sub_thing: FlatJustifiedSomeCustomType"),
            "Should flatten non-leaf type => sub_thing: FlatJustifiedSomeCustomType"
        );

        // Also expect top-level sub_thing_justification / sub_thing_confidence fields
        assert!(flat_string.contains("sub_thing_justification: String"));
        assert!(flat_string.contains("sub_thing_confidence: f32"));

        // item_inits => from(...) since skip_child_just=false
        assert_eq!(item_inits.len(), 1);
        let init_str = item_inits[0].to_string();
        debug!("item_inits => {}", init_str);
        assert!(
            init_str.contains(": :: core :: convert :: From :: from ( flat . sub_thing )"),
            "Expected from(...) usage for child flattening"
        );

        // justification & confidence => we must see child_just => Something like:
        //   sub_thing_justification: SomeCustomTypeJust { detail_justification: flat. ... } (or similar)
        // We see in code that we do `child_ty_to_just` => "SomeCustomTypeJustification" by default
        // or if it can't parse, fallback. We'll just check partial:
        let just_init_str = just_inits[0].to_string();
        let conf_init_str = conf_inits[0].to_string();

        debug!("just_inits[0] => {}", just_init_str);
        debug!("conf_inits[0] => {}", conf_init_str);
        assert!(just_init_str.contains("sub_thing_justification : SomeCustomTypeJustification {"));
        assert!(conf_init_str.contains("sub_thing_confidence : SomeCustomTypeConfidence {"));

        debug!("test_complex_field_nonleaf passed");
    }

    #[traced_test]
    fn test_unnamed_field_in_named_struct() {
        trace!("Starting test_unnamed_field_in_named_struct");
        // We'll parse a struct snippet that incorrectly has an unnamed field in a named struct context
        // e.g. struct Mixed { a: i32, bool, } => the second field is unnamed in the snippet => parse error
        // We'll simulate a partial parse or check how the code logs the warn and skip?

        // Actually, we'll do it by manual token approach:
        let item_struct: ItemStruct = parse_quote! {
            struct Mixed {
                named: String,
                // We'll try to pretend we have an unnamed field
                // parse_quote would fail if we truly wrote something like "bool," with no ident.
                // So let's just declare an actual named field but no ident => we can't parse that in normal Rust.
                // We'll confirm the existing code's skipping path is triggered if ident is None.
                #[allow(dead_code)]
                _phantom: bool
            }
        };

        // We'll simulate a scenario by forcibly removing the ident from the second field:
        let mut fields = match item_struct.fields {
            Fields::Named(ref named) => named.clone(),
            _ => panic!("Expected named fields"),
        };

        // Overwrite the second field's ident with None
        if fields.named.len() == 2 {
            let second = fields.named.iter_mut().nth(1).unwrap();
            second.ident = None;
        }

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            &fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        // Because the second field has ident=None => we skip it.
        // So we only flatten 'named' field
        let flat_string = flat_fields.iter().map(|ts| ts.to_string()).collect::<Vec<_>>().join("\n");
        info!("Flat expansions:\n{}", flat_string);

        // 'named' is a String => skip_self_just=false => skip_child_just=true => direct assignment
        // Expect "named: String", "named_justification: String", "named_confidence: f32"
        assert!(flat_string.contains("named: String"));
        assert!(flat_string.contains("named_justification: String"));
        assert!(flat_string.contains("named_confidence: f32"));

        // We skip the second field => no mention of it
        assert!(!flat_string.contains("_phantom"), "Should not see phantom in expansions");

        // item_inits => 1 entry for 'named'
        assert_eq!(item_inits.len(), 1);
        assert!(item_inits[0].to_string().contains("named: flat . named"));

        // just_inits/conf_inits => 1 each
        assert_eq!(just_inits.len(), 1);
        assert_eq!(conf_inits.len(), 1);

        debug!("test_unnamed_field_in_named_struct passed");
    }

    #[traced_test]
    fn test_error_during_flatten() {
        trace!("Starting test_error_during_flatten");
        // We'll force an error from compute_flat_type_for_stamped by using a "BadType" in the field
        // The code checks raw_str.contains("BadType") => returns an error => e.to_compile_error()
        let item_struct: ItemStruct = parse_quote! {
            struct ErrorProne {
                trouble: BadTypeHere
            }
        };

        let fields = match item_struct.fields {
            Fields::Named(ref named) => named,
            _ => panic!("Expected named fields"),
        };

        let mut flat_fields = Vec::new();
        let mut item_inits = Vec::new();
        let mut just_inits = Vec::new();
        let mut conf_inits = Vec::new();

        gather_flat_fields_and_inits_for_named(
            &item_struct.ident,
            fields,
            &mut flat_fields,
            &mut item_inits,
            &mut just_inits,
            &mut conf_inits,
        );

        // Expect an error in the expansions => compile_error
        let expansions_str = flat_fields.iter().map(|ts| ts.to_string()).collect::<Vec<_>>().join("\n");
        error!("Expansions:\n{}", expansions_str);

        // item_inits, just_inits, conf_inits => none, because we bail
        assert!(item_inits.is_empty(), "No item_inits expected after error");
        assert!(just_inits.is_empty(), "No just_inits expected after error");
        assert!(conf_inits.is_empty(), "No conf_inits expected after error");

        // Check that expansions_str has some compile_error
        assert!(
            expansions_str.contains("compile_error"),
            "Should contain compile_error in expansions"
        );

        debug!("test_error_during_flatten passed");
    }
}
