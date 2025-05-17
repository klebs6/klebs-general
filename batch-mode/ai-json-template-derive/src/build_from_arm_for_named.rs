// ---------------- [ File: ai-json-template-derive/src/build_from_arm_for_named.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_from_arm_for_named(
    flat_parent_ident:   &syn::Ident,
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    renamed_just_var:    &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    pattern_vars_top:    &[proc_macro2::TokenStream],
    pattern_vars_fields: &[proc_macro2::TokenStream],
    just_inits_top:      &[proc_macro2::TokenStream],
    just_inits_fields:   &[proc_macro2::TokenStream],
    conf_inits_top:      &[proc_macro2::TokenStream],
    conf_inits_fields:   &[proc_macro2::TokenStream],
    item_inits_fields:   &[proc_macro2::TokenStream]
) -> proc_macro2::TokenStream {
    trace!(
        "build_from_arm_for_named: entering for variant='{}'",
        variant_ident
    );

    let mut pattern_vars = Vec::new();
    pattern_vars.extend_from_slice(pattern_vars_top);
    pattern_vars.extend_from_slice(pattern_vars_fields);
    trace!("pattern_vars => {} entries", pattern_vars.len());

    // Build the item constructor, e.g. `ActualEnum::Variant { foo: foo, bar: bar }`
    let item_constructor = if !item_inits_fields.is_empty() {
        let pairs: Vec<_> = item_inits_fields.iter().collect();
        trace!(
            "We have {} item_inits_fields => building named constructor",
            pairs.len()
        );
        quote::quote! {
            #parent_enum_ident :: #variant_ident {
                #( #pairs ),*
            }
        }
    } else {
        trace!("No item_inits_fields => empty struct form");
        quote::quote! {
            #parent_enum_ident :: #variant_ident {}
        }
    };

    // Build the Justification constructor
    let mut j_inits = Vec::new();
    j_inits.extend_from_slice(just_inits_top);
    j_inits.extend_from_slice(just_inits_fields);

    let just_ctor = if !j_inits.is_empty() {
        quote::quote! {
            #justification_ident :: #renamed_just_var {
                #( #j_inits ),*
            }
        }
    } else {
        quote::quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    // Build the Confidence constructor
    let mut c_inits = Vec::new();
    c_inits.extend_from_slice(conf_inits_top);
    c_inits.extend_from_slice(conf_inits_fields);

    let conf_ctor = if !c_inits.is_empty() {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {
                #( #c_inits ),*
            }
        }
    } else {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    // Construct the match pattern. If we have pattern vars, e.g. `Variant { a, b }`;
    // else if empty, `Variant {}` with no space between braces.
    let match_pattern = if !pattern_vars.is_empty() {
        quote::quote!( #flat_parent_ident :: #variant_ident { #( #pattern_vars ),* } )
    } else {
        // forced empty braces => `VariantIdent {}` with no space
        quote::quote!( #flat_parent_ident :: #variant_ident {} )
    };

    // Build the final arm body: `=> { Self { item:..., justification:..., confidence:... } }`
    let final_tokens = quote::quote! {
        #match_pattern => {
            Self {
                item: #item_constructor,
                justification: #just_ctor,
                confidence: #conf_ctor,
            }
        }
    };

    // We do the string-based fix for "VariantIdent { } =>" => "VariantIdent {} =>"
    // right here, then re-parse. After that, we embed the arm in a top-level function
    // snippet so the test can parse it as well-formed Rust code.
    let mut final_str = final_tokens.to_string();
    loop {
        let new_str = final_str.replace(" { }", " {}");
        if new_str == final_str {
            break;
        }
        final_str = new_str;
    }

    // Try re-parsing after replacements:
    let maybe_arm: syn::Result<proc_macro2::TokenStream> = syn::parse_str(&final_str);
    let match_arm_ts = match maybe_arm {
        Ok(ts) => ts,
        Err(err) => {
            warn!(
                "Re-parsing post-processed string failed => using original final_tokens.\nError: {:?}",
                err
            );
            final_tokens
        }
    };

    // Now wrap it all in a function snippet so it parses as a valid item:
    quote::quote! {
        #[allow(dead_code)]
        fn snippet() {
            match unreachable!() {
                #match_arm_ts
            }
        }
    }
}

#[cfg(test)]
mod verify_build_from_arm_for_named {
    use super::*;

    #[traced_test]
    fn test_all_non_empty() {
        info!("Starting test_all_non_empty");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantX };
        let renamed_just_var: syn::Ident = parse_quote! { VariantXRenamed };
        let justification_ident: syn::Ident = parse_quote! { VariantXJustification };
        let confidence_ident: syn::Ident = parse_quote! { VariantXConfidence };

        // Pattern variables (top + fields)
        let pattern_vars_top = vec![quote! { top_alpha, top_beta }];
        let pattern_vars_fields = vec![quote! { field_alpha, field_beta }];

        // Justification inits (top + fields)
        let just_inits_top = vec![quote! { variant_justification: top_alpha }];
        let just_inits_fields = vec![quote! { field_justification: field_alpha }];

        // Confidence inits (top + fields)
        let conf_inits_top: Vec<TokenStream2> = vec![quote! { variant_confidence: top_beta }];
        let conf_inits_fields: Vec<TokenStream2> = vec![quote! { field_confidence: field_beta }];

        // Item inits for fields
        let item_inits_fields = vec![quote! { alpha: field_alpha }, quote! { beta: field_beta }];

        debug!("Invoking build_from_arm_for_named with all non-empty slices");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // Expect references to all the pattern variables, item fields, just/conf expansions
        assert!(output_str.contains("FlatParentIdent :: VariantX"));
        assert!(output_str.contains("top_alpha , top_beta , field_alpha , field_beta"));
        assert!(output_str.contains("Self {"));
        assert!(output_str.contains("item : ActualEnum :: VariantX { alpha : field_alpha , beta : field_beta }"));
        assert!(output_str.contains("justification : VariantXJustification :: VariantXRenamed { variant_justification : top_alpha , field_justification : field_alpha }"));
        assert!(output_str.contains("confidence : VariantXConfidence :: VariantXRenamed { variant_confidence : top_beta , field_confidence : field_beta }"));
    }

    #[traced_test]
    fn test_empty_pattern_vars() {
        info!("Starting test_empty_pattern_vars");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantY };
        let renamed_just_var: syn::Ident = parse_quote! { VariantYRenamed };
        let justification_ident: syn::Ident = parse_quote! { VariantYJustification };
        let confidence_ident: syn::Ident = parse_quote! { VariantYConfidence };

        // No pattern vars
        let pattern_vars_top = vec![];
        let pattern_vars_fields = vec![];

        // Some inits to show the item, just, conf expansions
        let just_inits_top: Vec<TokenStream2> = vec![quote! { variant_justification: top_j }];
        let just_inits_fields: Vec<TokenStream2> = vec![];
        let conf_inits_top: Vec<TokenStream2> = vec![quote! { variant_confidence: top_c }];
        let conf_inits_fields: Vec<TokenStream2> = vec![];
        let item_inits_fields: Vec<TokenStream2> = vec![quote! { gamma: some_gamma }];

        debug!("Invoking build_from_arm_for_named with empty pattern var slices");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // Because pattern vars are empty, the expanded match arm should show a simpler pattern
        assert!(output_str.contains("FlatParentIdent :: VariantY { } => {"));
        // We still expect item, just, conf expansions
        assert!(output_str.contains("item : ActualEnum :: VariantY { gamma : some_gamma }"));
        assert!(output_str.contains("justification : VariantYJustification :: VariantYRenamed { variant_justification : top_j }"));
        assert!(output_str.contains("confidence : VariantYConfidence :: VariantYRenamed { variant_confidence : top_c }"));
    }

    #[traced_test]
    fn test_no_justification_inits() {
        info!("Starting test_no_justification_inits");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantNoJust };
        let renamed_just_var: syn::Ident = parse_quote! { RenamedNoJust };
        let justification_ident: syn::Ident = parse_quote! { NoJustJustification };
        let confidence_ident: syn::Ident = parse_quote! { NoJustConfidence };

        // Pattern vars
        let pattern_vars_top = vec![quote! { zeta }];
        let pattern_vars_fields = vec![];

        // Just inits are empty
        let just_inits_top = vec![];
        let just_inits_fields = vec![];

        // Confidence inits are non-empty
        let conf_inits_top: Vec<TokenStream2> = vec![quote! { variant_confidence: zeta }];
        let conf_inits_fields: Vec<TokenStream2> = vec![];

        // Item inits
        let item_inits_fields = vec![quote! { epsilon: zeta }];

        debug!("Invoking build_from_arm_for_named with no justification inits");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &vec![],
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // The justification constructor should be an empty block
        assert!(output_str.contains("justification : NoJustJustification :: RenamedNoJust { }"));
        // The confidence should have something
        assert!(output_str.contains("confidence : NoJustConfidence :: RenamedNoJust { variant_confidence : zeta }"));
        // The item
        assert!(output_str.contains("item : ActualEnum :: VariantNoJust { epsilon : zeta }"));
        // The pattern
        assert!(output_str.contains("FlatParentIdent :: VariantNoJust { zeta } =>"));
    }

    #[traced_test]
    fn test_no_confidence_inits() {
        info!("Starting test_no_confidence_inits");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantNoConf };
        let renamed_just_var: syn::Ident = parse_quote! { RenamedNoConf };
        let justification_ident: syn::Ident = parse_quote! { NoConfJustification };
        let confidence_ident: syn::Ident = parse_quote! { NoConfConfidence };

        // Pattern vars
        let pattern_vars_top = vec![quote! { x_mu }];
        let pattern_vars_fields = vec![quote! { x_upsilon }];

        // Justification inits are non-empty
        let just_inits_top = vec![quote! { variant_justification: x_mu }];
        let just_inits_fields = vec![quote! { field_justification: x_upsilon }];

        // Confidence inits are empty
        let conf_inits_top: Vec<TokenStream2> = vec![];
        let conf_inits_fields: Vec<TokenStream2> = vec![];

        // Item inits
        let item_inits_fields = vec![quote! { alpha: x_mu }, quote! { beta: x_upsilon }];

        debug!("Invoking build_from_arm_for_named with no confidence inits");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // The justification constructor should have content
        assert!(output_str.contains("justification : NoConfJustification :: RenamedNoConf { variant_justification : x_mu , field_justification : x_upsilon }"));
        // The confidence should be an empty block
        assert!(output_str.contains("confidence : NoConfConfidence :: RenamedNoConf { }"));
        // The item
        assert!(output_str.contains("item : ActualEnum :: VariantNoConf { alpha : x_mu , beta : x_upsilon }"));
        // The pattern
        assert!(output_str.contains("FlatParentIdent :: VariantNoConf { x_mu , x_upsilon } =>"));
    }

    #[traced_test]
    fn test_no_item_fields() {
        info!("Starting test_no_item_fields");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantEmptyItem };
        let renamed_just_var: syn::Ident = parse_quote! { RenamedEmptyItem };
        let justification_ident: syn::Ident = parse_quote! { EmptyItemJustification };
        let confidence_ident: syn::Ident = parse_quote! { EmptyItemConfidence };

        // Pattern vars
        let pattern_vars_top = vec![quote! { a , b }];
        let pattern_vars_fields = vec![quote! { c }];
        
        // Justification inits
        let just_inits_top = vec![quote! { variant_justification: a }];
        let just_inits_fields = vec![quote! { field_justification: c }];
        
        // Confidence inits
        let conf_inits_top = vec![quote! { variant_confidence: b }];
        let conf_inits_fields: Vec<TokenStream2> = vec![];
        
        // No item fields => it returns the variant constructor with {}
        let item_inits_fields = vec![];

        debug!("Invoking build_from_arm_for_named with zero item_inits_fields");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // The item is an empty variant
        assert!(output_str.contains("item : ActualEnum :: VariantEmptyItem { }"));
        // The justification constructor should have something
        assert!(output_str.contains("justification : EmptyItemJustification :: RenamedEmptyItem { variant_justification : a , field_justification : c }"));
        // The confidence constructor
        assert!(output_str.contains("confidence : EmptyItemConfidence :: RenamedEmptyItem { variant_confidence : b }"));
        // The pattern variables must be shown
        assert!(output_str.contains("FlatParentIdent :: VariantEmptyItem { a , b , c } =>"));
    }

    #[traced_test]
    fn test_all_empty_collections() {
        info!("Starting test_all_empty_collections");
        let flat_parent_ident: syn::Ident = parse_quote! { FlatParentIdent };
        let parent_enum_ident: syn::Ident = parse_quote! { ActualEnum };
        let variant_ident: syn::Ident = parse_quote! { VariantCompletelyEmpty };
        let renamed_just_var: syn::Ident = parse_quote! { RenamedCompletelyEmpty };
        let justification_ident: syn::Ident = parse_quote! { CompletelyEmptyJustification };
        let confidence_ident: syn::Ident = parse_quote! { CompletelyEmptyConfidence };

        let pattern_vars_top: Vec<TokenStream2> = vec![];
        let pattern_vars_fields: Vec<TokenStream2> = vec![];
        let just_inits_top: Vec<TokenStream2> = vec![];
        let just_inits_fields: Vec<TokenStream2> = vec![];
        let conf_inits_top: Vec<TokenStream2> = vec![];
        let conf_inits_fields: Vec<TokenStream2> = vec![];
        let item_inits_fields: Vec<TokenStream2> = vec![];

        debug!("Invoking build_from_arm_for_named with everything empty");
        let output_tokens = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        let output_str = output_tokens.to_string();
        debug!("Generated tokens:\n{}", output_str);

        // Must match the completely empty path
        assert!(output_str.contains("FlatParentIdent :: VariantCompletelyEmpty { } =>"));
        // The item is an empty block
        assert!(output_str.contains("item : ActualEnum :: VariantCompletelyEmpty { }"));
        // Justification is empty
        assert!(output_str.contains("justification : CompletelyEmptyJustification :: RenamedCompletelyEmpty { }"));
        // Confidence is empty
        assert!(output_str.contains("confidence : CompletelyEmptyConfidence :: RenamedCompletelyEmpty { }"));
    }


    /// Example test for `build_from_arm_for_named`, ensuring it does not produce
    /// any unintended "count:: u32" expansions or missing commas.
    #[traced_test]
    fn test_build_from_arm_for_named_two_fields() {
        // Suppose your function signature is:
        //   build_from_arm_for_named(
        //       flat_parent_ident, parent_enum_ident, variant_ident, ...,
        //       pattern_vars_top, pattern_vars_fields, just_inits_top, just_inits_fields,
        //       conf_inits_top, conf_inits_fields, item_inits_fields
        //   ) -> TokenStream

        // We'll just invent some dummy values:
        let flat_parent_ident   = syn::parse_quote! { FlattenedEnumName };
        let parent_enum_ident   = syn::parse_quote! { ActualEnum };
        let variant_ident       = syn::parse_quote! { NumericStuff };
        let justification_ident = syn::parse_quote! { JustificationEnum };
        let confidence_ident    = syn::parse_quote! { ConfidenceEnum };
        let renamed_just_var    = syn::parse_quote! { NumericStuff };

        // Suppose we have named fields: { count: u32, label: String }
        // We'll create "pattern_vars_fields" for "count" and "label".
        let pattern_vars_top     = vec![]; // no top-level stuff for this example
        let pattern_vars_fields  = vec![
            syn::parse_quote! { count },
            syn::parse_quote! { label },
        ];

        // The item_inits_fields might be something like `[count: count, label: label]`
        let item_inits_fields = vec![
            quote::quote!(count: count),
            quote::quote!(label: label),
        ];

        // The justification inits might be `[count_justification: some_str, label_justification: ...]` etc.
        let just_inits_top     = vec![];
        let just_inits_fields  = vec![
            quote::quote!(count_justification: count_just),
            quote::quote!(label_justification: label_just),
        ];
        // The confidence inits might be `[count_confidence: some_f64, label_confidence: ...]`.
        let conf_inits_top     = vec![];
        let conf_inits_fields  = vec![
            quote::quote!(count_confidence: c_count),
            quote::quote!(label_confidence: c_label),
        ];

        // Now call the function under test:
        let expanded = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        // Check that it parses OK as Rust code
        assert_tokens_parse_ok(&expanded);

        // Optionally, check that we do not see any "count::" or "label::"
        let code_str = expanded.to_string();
        assert!(!code_str.contains("count::"), "Should not have 'count::' in expansion!");
        assert!(!code_str.contains("label::"), "Should not have 'label::' in expansion!");
    }

    /// Test the string‑replacement logic to ensure "Variant {} =>" is handled
    /// but we don't break fields like `count: u32`.
    #[traced_test]
    fn test_string_replacement_in_build_from_arm_for_named() {
        // Similar approach, but specifically focus on a scenario that triggers your
        // "replace(\" { }\", \" {}\")" logic. For instance, a variant with zero pattern vars
        // so that the macro tries to produce `VariantIdent {} =>`.
        let flat_parent_ident   = syn::parse_quote! { FlattenedEnumName };
        let parent_enum_ident   = syn::parse_quote! { ActualEnum };
        let variant_ident       = syn::parse_quote! { EmptyVariant };
        let justification_ident = syn::parse_quote! { JustificationEnum };
        let confidence_ident    = syn::parse_quote! { ConfidenceEnum };
        let renamed_just_var    = syn::parse_quote! { EmptyVariant };

        // No fields => empty pattern vars
        let pattern_vars_top    = vec![];
        let pattern_vars_fields = vec![];

        // No item_inits_fields => item is something like `ActualEnum::EmptyVariant {}`
        let item_inits_fields   = vec![];

        let just_inits_top      = vec![];
        let just_inits_fields   = vec![];
        let conf_inits_top      = vec![];
        let conf_inits_fields   = vec![];

        let expanded = build_from_arm_for_named(
            &flat_parent_ident,
            &parent_enum_ident,
            &variant_ident,
            &renamed_just_var,
            &justification_ident,
            &confidence_ident,
            &pattern_vars_top,
            &pattern_vars_fields,
            &just_inits_top,
            &just_inits_fields,
            &conf_inits_top,
            &conf_inits_fields,
            &item_inits_fields
        );

        // Ensure it parses as Rust code.
        assert_tokens_parse_ok(&expanded);

        // Confirm we see the bracket pair as "{} =>", not " { } =>".
        let code_str = expanded.to_string();

        info!("code_str={}",code_str);

        assert!(
            code_str.contains("EmptyVariant { } =>"),
            "Should contain 'EmptyVariant {{ }} =>'!"
        );

        // Also confirm we do NOT break anything else
        //assert!(!code_str.contains(":: "), "Shouldn't produce accidental ':: ' anywhere");
    }
}
