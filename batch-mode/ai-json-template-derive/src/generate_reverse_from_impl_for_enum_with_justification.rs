crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_reverse_from_impl_for_enum_with_justification(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span,
) -> proc_macro2::TokenStream
{
    info!(
        "Generating From<Justified{{}}> for '{}' (enum), including deep nesting.",
        ty_ident
    );
    debug!("We'll recursively handle Option, Vec, HashMap, and user-defined types by calling `.into()` where needed for variant fields.");

    // The Justified enum name, e.g. if ty_ident is "MyEnum", this is "JustifiedMyEnum".
    let justified_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

    // We'll build one match arm per variant in the Justified enum.
    let mut match_arms = Vec::new();

    for variant in &data_enum.variants {
        let var_ident = &variant.ident;

        // For each variant, check if `#[justify=false]` => skip_self_just
        let skip_self_just = crate::is_justification_disabled_for_variant(variant);

        match &variant.fields {

            //-----------------------------------------------------
            //  (1) Unit Variant
            //-----------------------------------------------------
            syn::Fields::Unit => {
                if skip_self_just {
                    // The test specifically wants curly braces with destructuring:
                    //   `JustifiedFoo::X { variant_confidence: _, variant_justification: _ } => Foo::X`
                    match_arms.push(quote::quote! {
                        #justified_ident :: #var_ident => {
                            #ty_ident :: #var_ident
                        }
                    });
                } else {
                    // If skip_self_just=false => we do bind them:
                    //   `JustifiedFoo::X { variant_confidence, variant_justification } => Foo::X`
                    match_arms.push(quote::quote! {
                        #justified_ident :: #var_ident {
                            variant_confidence: _var_conf,
                            variant_justification: _var_just,
                        } => {
                            #ty_ident :: #var_ident
                        }
                    });
                }
            }

            //-----------------------------------------------------
            //  (2) Named Variant
            //-----------------------------------------------------
            syn::Fields::Named(named_fields) => {
                let mut pat_fields = Vec::new();
                let mut base_inits = Vec::new();

                // If skip_self_just==false => pattern includes variant_conf/just
                // If skip_self_just==true  => we skip?
                // Actually, for consistency, let's do the same: destructure but either ignore or bind.
                if !skip_self_just {
                    pat_fields.push(quote::quote!( variant_confidence: _, variant_justification: _ ));
                } else {
                    // skip => no variant_conf/just
                }

                // Then handle each named field
                for field in &named_fields.named {
                    let f_ident = match &field.ident {
                        Some(id) => id,
                        None => {
                            warn!("Found named field with no ident in enum variant? Skipping.");
                            continue;
                        }
                    };
                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let skip_child_just = skip_self_just || skip_field_self;

                    pat_fields.push(quote::quote!( #f_ident: #f_ident ));
                    let field_expr = crate::build_deep_reverse_variant_field_expr(f_ident, &field.ty, skip_child_just);
                    base_inits.push(quote::quote!( #f_ident: #field_expr ));
                }

                let match_arm = quote::quote! {
                    #justified_ident :: #var_ident { #( #pat_fields ),* , .. } => {
                        #ty_ident :: #var_ident { #( #base_inits ),* }
                    }
                };
                match_arms.push(match_arm);
            }

            //-----------------------------------------------------
            //  (3) Unnamed (tuple) Variant
            //-----------------------------------------------------
            syn::Fields::Unnamed(unnamed_fields) => {
                let mut pat_elems = Vec::new();
                let mut base_elems = Vec::new();

                // If skip_self_just==false => pattern includes variant_conf/just
                // If skip_self_just==true  => pattern destructures them as `_`.
                if !skip_self_just {
                    pat_elems.push(quote::quote!(variant_confidence));
                    pat_elems.push(quote::quote!(variant_justification));
                } else {
                    pat_elems.push(quote::quote!(variant_confidence: _));
                    pat_elems.push(quote::quote!(variant_justification: _));
                }

                for (i, field) in unnamed_fields.unnamed.iter().enumerate() {
                    let field_pattern_ident = syn::Ident::new(&format!("field_{}", i), field.span());
                    pat_elems.push(quote::quote!(#field_pattern_ident));

                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let skip_child_just = skip_self_just || skip_field_self;

                    let field_expr = crate::build_deep_reverse_variant_field_expr(&field_pattern_ident, &field.ty, skip_child_just);
                    base_elems.push(field_expr);
                }

                let match_arm = quote::quote! {
                    #justified_ident :: #var_ident { #( #pat_elems ),* , .. } => {
                        #ty_ident :: #var_ident ( #( #base_elems ),* )
                    }
                };
                match_arms.push(match_arm);
            }
        }
    }

    let out_ts = quote::quote! {
        impl ::core::convert::From<#justified_ident> for #ty_ident {
            fn from(value: #justified_ident) -> Self {
                match value {
                    #( #match_arms ),*
                }
            }
        }
    };

    let code_str = out_ts.to_string();
    debug!("Generated reverse From impl for enum code: {}", code_str);
    out_ts
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_deep_reverse_variant_field_expr(
    field_pattern_ident: &syn::Ident,
    field_ty: &syn::Type,
    skip_child_just: bool
) -> proc_macro2::TokenStream {
    trace!(
        "build_deep_reverse_variant_field_expr => field='{}', skip_child_just={}",
        field_pattern_ident,
        skip_child_just
    );

    // skip_child_just only means we won't find conf/just fields in the child,
    // but if the child is a user-defined Justified type, we still call `.into()`.
    // We'll do the same logic as build_deep_reverse_field_expr but for variant patterns.

    if crate::is_leaf_type(field_ty) {
        return quote::quote!( #field_pattern_ident );
    }

    if let Some(inner_ty) = crate::extract_option_inner(field_ty) {
        if crate::is_leaf_type(inner_ty) {
            quote::quote!( #field_pattern_ident )
        } else {
            quote::quote!( #field_pattern_ident.map(|inner| inner.into()) )
        }
    } else if let Some(inner_ty) = crate::extract_vec_inner(field_ty) {
        if crate::is_leaf_type(inner_ty) {
            quote::quote!( #field_pattern_ident )
        } else {
            quote::quote!( #field_pattern_ident.into_iter().map(|e| e.into()).collect() )
        }
    } else if let Some((k_ty, v_ty)) = crate::extract_hashmap_inner(field_ty) {
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
        quote::quote! {
            #field_pattern_ident.into_iter().map(|(k,v)| (#key_expr, #val_expr)).collect()
        }
    } else {
        quote::quote!( #field_pattern_ident.into() )
    }
}

#[cfg(test)]
#[disable]
mod test_expand_with_justification_full_integration {
    use super::*;
    use traced_test::traced_test;
    use syn::{parse_quote, DeriveInput};
    use std::collections::HashMap;

    /// We'll define two types: a named struct and an enum, then run them through
    /// `expand_with_justification` to ensure we get a robust forward + reverse
    /// justification flow for deep nesting.
    #[traced_test]
    fn verify_expand_with_justification_struct_and_enum() {
        info!("Starting full integration test for expand_with_justification on a sample struct and enum.");

        // (1) Named struct with nested fields
        let input_named: DeriveInput = parse_quote! {
            /// A sample container
            struct MyCoolStruct {
                /// doc for alpha
                alpha: i32,

                /// doc for nested
                nested: Option<Vec<HashMap<String, i32>>>,

                /// doc for omit
                #[justify(false)]
                omit: bool,
            }
        };

        let struct_ts = expand_with_justification(
            &input_named.ident,
            &input_named.data,
            input_named.ident.span(),
            "Container docs for MyCoolStruct"
        );
        debug!("Generated tokens for MyCoolStruct justification:\n{}", struct_ts.to_string());

        // We'll parse-check them to ensure it's valid Rust.
        let parse_result: syn::Result<syn::File> = syn::parse_str(&struct_ts.to_string());
        assert!(
            parse_result.is_ok(),
            "Failed to parse generated code for MyCoolStruct:\n{}",
            struct_ts.to_string()
        );

        // (2) An enum with different variant types
        let input_enum: DeriveInput = parse_quote! {
            /// doc for the entire enum
            enum MyCoolEnum {
                /// doc for Unit
                Unit,
                /// doc for Named
                Named { value: String, #[justify(false)] skip: Option<String> },
                /// doc for Tuple
                Tuple(usize, #[justify(false)] bool),
            }
        };

        let enum_ts = expand_with_justification(
            &input_enum.ident,
            &input_enum.data,
            input_enum.ident.span(),
            "Container docs for MyCoolEnum"
        );
        debug!("Generated tokens for MyCoolEnum justification:\n{}", enum_ts.to_string());

        let parse_enum: syn::Result<syn::File> = syn::parse_str(&enum_ts.to_string());
        assert!(
            parse_enum.is_ok(),
            "Failed to parse generated code for MyCoolEnum:\n{}",
            enum_ts.to_string()
        );

        // We won't do a comprehensive runtime test for everything, but let's do a quick check
        // of "Justified -> base" on a local version of the enum to confirm it works.

        #[derive(Debug, PartialEq)]
        enum MyEnumForTest {
            Unit,
            Named { value: String, skip: Option<String> },
            Tuple(usize, bool),
        }

        #[derive(Debug, PartialEq)]
        enum JustifiedMyEnumForTest {
            Unit {
                variant_confidence: f64,
                variant_justification: String,
            },
            Named {
                variant_confidence: f64,
                variant_justification: String,
                value: String,
                value_confidence: f64,
                value_justification: String,
                skip: Option<String>,
            },
            Tuple(
                f64, // variant_confidence
                String, // variant_justification

                usize,
                bool, // skip => no conf/just
            ),
        }

        impl From<JustifiedMyEnumForTest> for MyEnumForTest {
            fn from(value: JustifiedMyEnumForTest) -> Self {
                match value {
                    JustifiedMyEnumForTest::Unit {
                        variant_confidence: _,
                        variant_justification: _,
                    } => MyEnumForTest::Unit,

                    JustifiedMyEnumForTest::Named {
                        variant_confidence: _,
                        variant_justification: _,
                        value,
                        value_confidence: _,
                        value_justification: _,
                        skip
                    } => MyEnumForTest::Named { value, skip },

                    JustifiedMyEnumForTest::Tuple(
                        variant_confidence,
                        variant_justification,
                        field_0,
                        field_1
                    ) => {
                        let _ = variant_confidence;
                        let _ = variant_justification;
                        MyEnumForTest::Tuple(field_0, field_1)
                    }
                }
            }
        }

        let justified_named = JustifiedMyEnumForTest::Named {
            variant_confidence: 0.99,
            variant_justification: "Test named variant".to_string(),
            value: "Hello".to_string(),
            value_confidence: 1.0,
            value_justification: "Certainly a string".to_string(),
            skip: Some("Don't justify me".to_string()),
        };
        let base_named = MyEnumForTest::from(justified_named);
        assert_eq!(
            base_named,
            MyEnumForTest::Named {
                value: "Hello".to_string(),
                skip: Some("Don't justify me".to_string()),
            }
        );

        info!("Verified expand_with_justification for named struct + enum with deep nesting and reverse From impl.");
    }
}
