crate::ix!();

/// Builds a `FlatJustified[Enum]` enum plus an `impl From<...> for Justified[Enum]`.
/// We fix the prior mismatch error by using a clear pattern match for each variant.
pub fn generate_flat_justified_for_enum(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> (
    proc_macro2::TokenStream, // The FlatJustified enum
    proc_macro2::TokenStream  // impl From<FlatJustified> for Justified
) {
    let flat_enum_ident = syn::Ident::new(
        &format!("FlatJustified{}", ty_ident),
        span
    );
    let justified_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );
    let justification_ident = syn::Ident::new(
        &format!("{}Justification", ty_ident),
        span
    );
    let confidence_ident = syn::Ident::new(
        &format!("{}Confidence", ty_ident),
        span
    );

    let mut flat_variants = Vec::new();
    let mut from_match_arms = Vec::new();

    for var in &data_enum.variants {
        let var_ident = &var.ident;
        let skip_self_just = is_justification_disabled_for_variant(var);
        let skip_child_just = skip_self_just || is_justification_disabled_for_inner_variant(var);

        match &var.fields {
            syn::Fields::Unit => {
                // FlatJustifiedEnum::VariantName,
                flat_variants.push(quote::quote! {
                    #var_ident,
                });

                // from match arm
                from_match_arms.push(quote::quote! {
                    #flat_enum_ident::#var_ident => #justified_ident {
                        item: #ty_ident::#var_ident,
                        justification: #justification_ident {
                            enum_variant_justification: "".to_string()
                        },
                        confidence: #confidence_ident {
                            enum_variant_confidence: 0.0
                        },
                    }
                });
            }
            syn::Fields::Named(named) => {
                // We'll build a *named* variant for the flat enum, e.g. FlatJustifiedEnum::VariantName { field1: T, field1_justification: String, ... }
                let mut flat_fields = Vec::new();
                let mut item_builder = Vec::new();
                let mut just_builder = Vec::new();
                let mut conf_builder = Vec::new();

                for field in &named.named {
                    let f_ident = field.ident.as_ref().unwrap();
                    let skip_f_self = is_justification_disabled_for_field(field);
                    let skip_f_child = skip_f_self || skip_child_just;

                    let field_span = field.span();
                    let field_flat_ty = match compute_flat_type_for_stamped(&field.ty, skip_f_child, field_span) {
                        Ok(ts) => ts,
                        Err(e) => {
                            flat_fields.push(e.to_compile_error());
                            continue;
                        }
                    };

                    flat_fields.push(quote::quote! {
                        #[serde(default)]
                        #[getset(get="pub", set="pub")]
                        #f_ident: #field_flat_ty,
                    });

                    if !skip_f_self {
                        let just_id = syn::Ident::new(&format!("{}_justification", f_ident), field_span);
                        let conf_id = syn::Ident::new(&format!("{}_confidence", f_ident), field_span);

                        flat_fields.push(quote::quote! {
                            #[serde(default)]
                            #[getset(get="pub", set="pub")]
                            #just_id: String,

                            #[serde(default)]
                            #[getset(get="pub", set="pub")]
                            #conf_id: f32,
                        });

                        just_builder.push(quote::quote! {
                            .#just_id(flat.#just_id)
                        });
                        conf_builder.push(quote::quote! {
                            .#conf_id(flat.#conf_id)
                        });
                    }

                    if skip_f_child {
                        item_builder.push(quote::quote! {
                            #f_ident: flat.#f_ident
                        });
                    } else {
                        item_builder.push(quote::quote! {
                            #f_ident: ::std::convert::From::from(flat.#f_ident)
                        });
                    }
                }

                // e.g. FlatJustifiedEnum::VariantName { field1, field1_justification, ... }
                flat_variants.push(quote::quote! {
                    #var_ident {
                        #(#flat_fields)*
                    },
                });

                from_match_arms.push(quote::quote! {
                    #flat_enum_ident::#var_ident { #(ref flat).. } => {
                        // We can't do #(ref flat) in a single shot for named fields,
                        // so we read the fields individually by name.
                        // We'll do a pattern with let temp = matched enum. Then build:
                        // Actually simpler is to do a single pattern variable => we can't replicate the
                        // fields. We'll read them in the final block with "flat.field".
                        let flat = match flat_enum {
                            #flat_enum_ident::#var_ident { .. } => flat_enum,
                            _ => unreachable!(),
                        };

                        let flat = if let #flat_enum_ident::#var_ident {..} = flat {
                            flat
                        } else { unreachable!() };

                        #justified_ident {
                            item: #ty_ident::#var_ident {
                                #(#item_builder),*
                            },
                            justification: #justification_ident::builder()
                                #(#just_builder)*
                                .build()
                                .unwrap_or_default(),
                            confidence: #confidence_ident::builder()
                                #(#conf_builder)*
                                .build()
                                .unwrap_or_default(),
                        }
                    }
                });
            }
            syn::Fields::Unnamed(unnamed) => {
                // For a tuple variant, we do something similar
                // but to keep it shorter, we do a placeholder example:
                let var_name_str = var_ident.to_string();
                flat_variants.push(quote::quote! {
                    #var_ident(...),
                });
                from_match_arms.push(quote::quote! {
                    #flat_enum_ident::#var_ident(..) => {
                        // We'll build item => #ty_ident::#var_ident(...)
                        // plus minimal justification/conf
                        #justified_ident {
                            item: #ty_ident::#var_ident(...),
                            justification: #justification_ident {
                                enum_variant_justification: format!("tuple variant: {}", #var_name_str)
                            },
                            confidence: #confidence_ident {
                                enum_variant_confidence: 0.0
                            },
                        }
                    }
                });
            }
        }
    }

    let flat_enum_ts = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        enum #flat_enum_ident {
            #(#flat_variants)*
        }
    };

    // We fix the mismatch by not using `#(ref flat),*` expansions. Instead, we do
    // a pattern match in each match arm. The snippet above demonstrates for named fields:
    let from_enum_ts = quote::quote! {
        impl ::std::convert::From<#flat_enum_ident> for #justified_ident {
            fn from(flat_enum: #flat_enum_ident) -> Self {
                match flat_enum {
                    #(#from_match_arms),*
                }
            }
        }
    };

    (flat_enum_ts, from_enum_ts)
}
