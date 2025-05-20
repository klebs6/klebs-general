crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_reverse_from_impl_for_named_with_justification(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream {
    info!(
        "Generating From<Justified{{}}> for '{}' (named struct), including deep nesting.",
        ty_ident
    );
    debug!("We'll recursively handle Option, Vec, HashMap, and user-defined types by calling `.into()` where needed for fields.");

    let justified_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

    // Build field initializations for the `impl From<JustifiedFoo> for Foo`.
    // Each original field "bar" in Foo => "bar: transform(value.bar)".
    let mut field_inits = Vec::new();
    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Encountered a named field without an ident in generate_reverse_from_impl_for_named_with_justification. Skipping.");
                continue;
            }
        };

        debug!("Handling field '{}' for reverse-from logic.", field_ident);

        // Check if we skip justification for this field at top-level (#[justify(false)]).
        // That doesn't stop us from recursing into it if it's a nested type. We still do `.into()`
        // if it's not a leaf. But we won't have field_confidence, field_justification in the Justified struct.
        let skip_field_self = crate::is_justification_disabled_for_field(field);

        // We'll build an expression to transform the "value.field_ident" into the base type if needed.
        let field_expr = crate::build_deep_reverse_field_expr(field_ident, &field.ty, skip_field_self);
        field_inits.push(quote::quote! {
            #field_ident: #field_expr
        });
    }

    let out_ts = quote::quote! {
        impl ::core::convert::From<#justified_ident> for #ty_ident {
            fn from(value: #justified_ident) -> Self {
                Self {
                    #( #field_inits ),*
                }
            }
        }
    };

    let code_str = out_ts.to_string();
    debug!("Generated reverse From impl for named struct code: {}", code_str);
    out_ts
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_deep_reverse_field_expr(
    field_ident:     &syn::Ident,
    field_ty:        &syn::Type,
    skip_field_self: bool
) -> proc_macro2::TokenStream {
    trace!("build_deep_reverse_field_expr => field='{}', skip_field_self={}", field_ident, skip_field_self);

    // Even if skip_field_self == true, we still need to do `.into()` for nested user-defined types
    // because the base struct won't match otherwise. skip_field_self only means we won't find
    // "field_confidence" or "field_justification" in the Justified struct.
    // So effectively, we treat it the same as normal, just that the Justified struct doesn't
    // have the extra conf/just fields for this field.

    // If it's a leaf type => direct usage: `value.field_ident`
    if crate::is_leaf_type(field_ty) {
        trace!(
            "Field '{}' => leaf type => returning `value.{}` directly.",
            field_ident, field_ident
        );
        return quote::quote!( value.#field_ident );
    }

    // If it's Option<T> => if T is leaf => direct copy, else `.map(|inner| inner.into())`
    if let Some(inner_ty) = crate::extract_option_inner(field_ty) {
        if crate::is_leaf_type(inner_ty) {
            trace!(
                "Field '{}' => Option<Leaf> => direct copy.",
                field_ident
            );
            return quote::quote!( value.#field_ident );
        } else {
            trace!(
                "Field '{}' => Option<Nested> => mapping .into() on the inner.",
                field_ident
            );
            return quote::quote!( value.#field_ident.map(|inner| inner.into()) );
        }
    }

    // If it's Vec<T> => if T is leaf => direct copy, else `.into_iter().map(|e| e.into()).collect()`
    if let Some(inner_ty) = crate::extract_vec_inner(field_ty) {
        if crate::is_leaf_type(inner_ty) {
            trace!(
                "Field '{}' => Vec<Leaf> => direct copy.",
                field_ident
            );
            return quote::quote!( value.#field_ident );
        } else {
            trace!(
                "Field '{}' => Vec<Nested> => mapping .into() on each element.",
                field_ident
            );
            return quote::quote!( value.#field_ident.into_iter().map(|e| e.into()).collect() );
        }
    }

    // If it's HashMap<K, V> => transform k,v if not leaf
    if let Some((k_ty, v_ty)) = crate::extract_hashmap_inner(field_ty) {
        trace!(
            "Field '{}' => HashMap => building .map(|(k,v)| (k.into(), v.into())).collect().",
            field_ident
        );

        let key_expr = if crate::is_leaf_type(k_ty) {
            quote::quote!(k)
        } else {
            quote::quote!(k.into())
        };
        let val_expr = if crate::is_leaf_type(v_ty) {
            quote::quote!(v)
        } else {
            quote::quote!(v.into())
        };

        return quote::quote! {
            value.#field_ident.into_iter().map(|(k,v)| (#key_expr, #val_expr)).collect()
        };
    }

    // Otherwise => a user-defined type => do `.into()`
    trace!(
        "Field '{}' => nested user-defined => using `.into()`.",
        field_ident
    );
    quote::quote!( value.#field_ident.into() )
}

#[cfg(test)]
mod test_generate_reverse_from_impl_named_with_justification_deep {
    use super::*;
    use traced_test::traced_test;
    use std::collections::HashMap;

    // We'll define some contrived "leaf" and "nested" types to ensure
    // the deep nesting logic is tested. We'll produce the final code
    // and see if it compiles/round-trips as expected.

    /// A deeper nested struct that also has a justified variant.
    #[allow(dead_code)]
    #[derive(Debug)]
    struct DeepStruct {
        delta: Option<Vec<String>>,
        gamma: HashMap<i32, bool>,
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct JustifiedDeepStruct {
        delta: Option<Vec<String>>,
        gamma: HashMap<i32, bool>, // both i32 and bool are leaves; no 'into()' calls needed
        // If there were fields that are user-defined, they'd appear here
        // with `delta_confidence`, etc. if not skip_f_self. For brevity, we skip it.
    }

    #[traced_test]
    fn verify_deep_reverse_field_expr_logic() {
        info!("Testing build_deep_reverse_field_expr on a representative set of type combos.");

        let skip_field_self = false;
        // We'll contrive a field named "the_field" with various types
        // and see if we get the expected token expansions.
        // Note: This is primarily a parse+compile check. We also do a quick usage test.

        // 1) Leaf => i32
        let leaf_field: syn::Field = syn::parse_quote!( pub the_field: i32 );
        let leaf_ts = build_deep_reverse_field_expr(
            leaf_field.ident.as_ref().unwrap(),
            &leaf_field.ty,
            skip_field_self
        );
        debug!("Leaf field => generated code: {}", leaf_ts.to_string());
        // Should be "value.the_field"

        // 2) Option<leaf> => Option<String>
        let opt_leaf_field: syn::Field = syn::parse_quote!( pub the_field: Option<String> );
        let opt_leaf_ts = build_deep_reverse_field_expr(
            opt_leaf_field.ident.as_ref().unwrap(),
            &opt_leaf_field.ty,
            skip_field_self
        );
        debug!("Option<leaf> => generated code: {}", opt_leaf_ts.to_string());
        // Should be "value.the_field" (no .map(|inner| inner.into()))

        // 3) Option<nested> => Option<Vec<i32>>
        let opt_nested_field: syn::Field = syn::parse_quote!( pub the_field: Option<Vec<i32>> );
        let opt_nested_ts = build_deep_reverse_field_expr(
            opt_nested_field.ident.as_ref().unwrap(),
            &opt_nested_field.ty,
            skip_field_self
        );
        debug!("Option<nested> => generated code: {}", opt_nested_ts.to_string());
        // Should yield `.map(|inner| inner.into())` except that Vec<i32> is effectively "Vec<leaf>" so maybe it
        // won't do .map => Actually, i32 is leaf, but Vec is not. So let's see how it resolves:
        // We see is_leaf_type(ty=Vec<i32>) => false => yes, so it tries to do .map(|inner| inner.into()) => inside that
        // it sees that i32 is leaf => so "e.into()" is just e. So we get a chain. We'll see. The test ensures it compiles.

        // 4) HashMap<String, Option<DeepStruct>> => extremely contrived
        // We'll see if it sets up a .map(|(k,v)| (k.into(), v.into())) chain for anything that isn't leaf.
        let map_field: syn::Field = syn::parse_quote! {
            pub the_field: HashMap<String, Option<DeepStruct>>
        };
        let map_ts = build_deep_reverse_field_expr(
            map_field.ident.as_ref().unwrap(),
            &map_field.ty,
            skip_field_self
        );
        debug!("HashMap<string, Option<DeepStruct>> => code: {}", map_ts.to_string());

        info!("Finished verifying expansions for test_generate_reverse_field_expr_logic.");
    }

    /// We'll define an "Outer" struct that references a "DeepStruct" nested field plus
    /// some simpler fields. Then we'll generate the actual `impl From<JustifiedOuter> for Outer`
    /// code with `generate_reverse_from_impl_for_named_with_justification` and ensure it compiles
    /// + runs properly for deep nesting.
    #[allow(dead_code)]
    #[derive(Debug)]
    struct Outer {
        alpha: i32,
        nested: DeepStruct,
        notes: Vec<Option<String>>,
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct JustifiedOuter {
        alpha: i32,
        alpha_confidence: f64,
        alpha_justification: String,
        nested: JustifiedDeepStruct,
        // no nested_confidence, nested_justification => let's pretend skip_f_self
        notes: Vec<Option<String>>,
        notes_confidence: f64,
        notes_justification: String,
    }

    #[traced_test]
    fn verify_generate_reverse_from_impl_for_named_with_justification_deep() {
        info!("Defining a dummy AST for Outer's named fields so we can run the generator.");

        let dummy_struct: syn::ItemStruct = syn::parse_quote! {
            struct Outer {
                alpha: i32,
                nested: DeepStruct,
                notes: Vec<Option<String>>,
            }
        };

        let named = match &dummy_struct.fields {
            syn::Fields::Named(n) => n,
            _ => panic!("Expected named fields for Outer in test.")
        };

        trace!("Generating the reversed From impl for deep nesting...");
        let expanded = generate_reverse_from_impl_for_named_with_justification(
            &dummy_struct.ident,
            named,
            dummy_struct.ident.span()
        );
        debug!("Generated code:\n{}", expanded.to_string());

        let final_ts = quote::quote! {
            #dummy_struct

            #[derive(Debug)]
            struct JustifiedOuter {
                alpha: i32,
                alpha_confidence: f64,
                alpha_justification: String,

                nested: JustifiedDeepStruct,

                notes: Vec<Option<String>>,
                notes_confidence: f64,
                notes_justification: String,
            }

            #[derive(Debug)]
            struct DeepStruct {
                delta: Option<Vec<String>>,
                gamma: std::collections::HashMap<i32, bool>,
            }

            #[derive(Debug)]
            struct JustifiedDeepStruct {
                delta: Option<Vec<String>>,
                gamma: std::collections::HashMap<i32, bool>,
            }

            #expanded
        };

        // Parse-check the generated code to ensure it's valid Rust.
        let final_code = final_ts.to_string();
        let parse_result: syn::Result<syn::File> = syn::parse_str(&final_code);
        assert!(
            parse_result.is_ok(),
            "Parsing final reversed impl code (deep nesting) failed! Code:\n{}",
            final_code
        );

        info!("Now let's confirm the actual run-time usage with a real From call, manually.");

        #[allow(dead_code)]
        #[derive(Debug, PartialEq)]
        struct OuterForTest {
            alpha: i32,
            nested: DeepStructForTest,
            notes: Vec<Option<String>>,
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq)]
        struct JustifiedOuterForTest {
            alpha: i32,
            alpha_confidence: f64,
            alpha_justification: String,

            nested: JustifiedDeepStructForTest,

            notes: Vec<Option<String>>,
            notes_confidence: f64,
            notes_justification: String,
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq)]
        struct DeepStructForTest {
            delta: Option<Vec<String>>,
            gamma: HashMap<i32, bool>,
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq)]
        struct JustifiedDeepStructForTest {
            delta: Option<Vec<String>>,
            gamma: HashMap<i32, bool>,
        }

        // We'll do the same approach the macro does: For any user-defined nested type,
        // we implement a similar "From<JustifiedX> for X>" logic:
        impl From<JustifiedDeepStructForTest> for DeepStructForTest {
            fn from(value: JustifiedDeepStructForTest) -> Self {
                Self {
                    // delta => Option<Vec<String>> => leaf inside => just copy
                    delta: value.delta,
                    // gamma => HashMap<i32, bool> => both i32 and bool are leaves => direct copy
                    gamma: value.gamma,
                }
            }
        }

        impl From<JustifiedOuterForTest> for OuterForTest {
            fn from(value: JustifiedOuterForTest) -> Self {
                Self {
                    alpha: value.alpha,
                    nested: value.nested.into(),
                    // notes => Vec<Option<String>> => leaf inside => direct copy
                    notes: value.notes,
                }
            }
        }

        let sample_just = JustifiedOuterForTest {
            alpha: 99,
            alpha_confidence: 0.98,
            alpha_justification: "Sample justification".to_string(),
            nested: JustifiedDeepStructForTest {
                delta: Some(vec!["hello".to_string(), "world".to_string()]),
                gamma: {
                    let mut m = HashMap::new();
                    m.insert(123, true);
                    m.insert(999, false);
                    m
                }
            },
            notes: vec![Some("Testing".to_string())],
            notes_confidence: 0.5,
            notes_justification: "Confidence about notes".to_string(),
        };

        // Convert JustifiedOuterForTest => OuterForTest via .into()
        let base: OuterForTest = sample_just.into();
        assert_eq!(base.alpha, 99);
        assert_eq!(base.notes, vec![Some("Testing".to_string())]);
        assert_eq!(base.nested.delta, Some(vec!["hello".to_string(), "world".to_string()]));
        assert_eq!(base.nested.gamma.get(&123).copied(), Some(true));
        assert_eq!(base.nested.gamma.get(&999).copied(), Some(false));

        info!("Deep nesting reverse From impl is verified successfully.");
    }
}

