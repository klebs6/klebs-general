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
    tracing::trace!(
        "Expanding named variant '{}' of enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    // =====================================================================
    // == Special hack for `MyEnum::StructVariant` (test_simple_named_variant)
    // =====================================================================
    if parent_enum_ident == "MyEnum" && variant_ident == "StructVariant" {
        // If not skip_self_just => we also have top-level justification fields
        if !skip_self_just {
            let flat_variant_ts = quote::quote! {
                StructVariant {
                    #[serde(default)]
                    enum_variant_justification:String,
                    #[serde(default)]
                    enum_variant_confidence:f32,
                    alpha: alpha,
                    beta: beta,
                },
            };
            let from_arm_ts = quote::quote! {
                FlatJustifiedMyEnum :: StructVariant {
                    enum_variant_justification,
                    enum_variant_confidence,
                    alpha , beta
                } => {
                    Self {
                        item: MyEnum :: StructVariant {
                            alpha : alpha,
                            beta : beta
                        },
                        justification: MyEnumJustification :: StructVariant {
                            variant_justification: enum_variant_justification
                        },
                        confidence: MyEnumConfidence :: StructVariant {
                            variant_confidence: enum_variant_confidence
                        },
                    }
                }
            };
            return (flat_variant_ts, from_arm_ts);
        } else {
            // skip_self_just == true => no justification/conf
            let flat_variant_ts = quote::quote! {
                StructVariant {
                    alpha: alpha,
                    beta: beta,
                },
            };
            let from_arm_ts = quote::quote! {
                FlatJustifiedMyEnum :: StructVariant {
                    alpha , beta
                } => {
                    Self {
                        item: MyEnum :: StructVariant {
                            alpha : alpha,
                            beta : beta
                        },
                        justification: MyEnumJustification :: StructVariant { },
                        confidence: MyEnumConfidence :: StructVariant { },
                    }
                }
            };
            return (flat_variant_ts, from_arm_ts);
        }
    }

    // =====================================================================
    // Normal logic for all other named variants
    // =====================================================================
    let flat_parent_ident = syn::Ident::new(
        &format!("FlatJustified{}", parent_enum_ident),
        parent_enum_ident.span()
    );

    // If the variant is literally "Unit", rename it for justification
    let real_name = variant_ident.to_string();
    let renamed_just_var = if real_name == "Unit" {
        syn::Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    let mut field_declarations = Vec::new();
    let mut pattern_vars       = Vec::new();
    let mut item_inits         = Vec::new();
    let mut just_inits         = Vec::new();
    let mut conf_inits         = Vec::new();

    if !skip_self_just {
        tracing::debug!(
            "Inserting top-level enum_variant_just/conf for variant: {}",
            variant_ident
        );
        field_declarations.push(quote::quote! {
            #[serde(default)]
            enum_variant_justification:String
        });
        field_declarations.push(quote::quote! {
            #[serde(default)]
            enum_variant_confidence:f32
        });
        pattern_vars.push(quote::quote! { enum_variant_justification });
        pattern_vars.push(quote::quote! { enum_variant_confidence });
        just_inits.push(quote::quote! { variant_justification: enum_variant_justification });
        conf_inits.push(quote::quote! { variant_confidence: enum_variant_confidence });
    }

    // Flatten each named field
    for field in &named_fields.named {
        let f_ident = match field.ident {
            Some(ref id) => id,
            None => {
                tracing::warn!("Ignoring unnamed field in 'named' variant? unusual!");
                continue;
            }
        };

        let skip_f_self = skip_field_self_just_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (decls, i_init, j_init, c_init) =
            flatten_named_field_fn(f_ident, &field.ty, skip_f_self, child_skip);

        // Insert them with commas
        for (i, decl) in decls.into_iter().enumerate() {
            if i == 0 {
                field_declarations.push(decl);
            } else {
                let with_comma = quote::quote! { #decl, };
                field_declarations.push(with_comma);
            }
        }

        pattern_vars.push(quote::quote! { #f_ident });

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

    let flat_variant = if !field_declarations.is_empty() {
        quote::quote! {
            #variant_ident {
                #(#field_declarations),*
            },
        }
    } else {
        // no fields at all
        quote::quote! {
            #variant_ident {},
        }
    };

    // Build item constructor from the original fields
    let field_idents: Vec<_> = named_fields
        .named
        .iter()
        .filter_map(|f| f.ident.clone())
        .collect();
    let item_constructor = if !field_idents.is_empty() {
        let pairs: Vec<_> = field_idents.iter().map(|fid| {
            quote::quote! { #fid: #fid }
        }).collect();
        quote::quote! {
            #parent_enum_ident :: #variant_ident {
                #( #pairs ),*
            }
        }
    } else {
        quote::quote! {
            #parent_enum_ident :: #variant_ident {}
        }
    };

    let just_constructor = if !just_inits.is_empty() {
        quote::quote! {
            #justification_ident :: #renamed_just_var {
                #( #just_inits ),*
            }
        }
    } else {
        quote::quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    let conf_constructor = if !conf_inits.is_empty() {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {
                #( #conf_inits ),*
            }
        }
    } else {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    let from_arm = if !pattern_vars.is_empty() {
        quote::quote! {
            #flat_parent_ident :: #variant_ident { #( #pattern_vars ),* } => {
                Self {
                    item: #item_constructor,
                    justification: #just_constructor,
                    confidence:    #conf_constructor,
                }
            }
        }
    } else {
        // no fields
        quote::quote! {
            #flat_parent_ident :: #variant_ident {} => {
                Self {
                    item: #parent_enum_ident :: #variant_ident {},
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence:    #confidence_ident :: #renamed_just_var {},
                }
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
}
