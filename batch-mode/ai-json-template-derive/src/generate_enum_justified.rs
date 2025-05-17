// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream {
    use quote::quote;

    // e.g. "Geometry2dWithJustification"
    let enum_with_just_ident = syn::Ident::new(
        &format!("{}WithJustification", ty_ident),
        span
    );

    // e.g. "JustifiedGeometry2d"
    let justified_enum_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

    // Detect if the user’s original enum has a `#[default]` variant
    // so we can safely derive Default for the expanded “WithJustification” enum.
    let mut user_has_default_variant = false;
    for variant in &data_enum.variants {
        for attr in &variant.attrs {
            if attr.path().is_ident("default") {
                user_has_default_variant = true;
                break;
            }
        }
        if user_has_default_variant {
            break;
        }
    }

    let mut variant_defs: Vec<TokenStream2> = Vec::new();

    for variant in &data_enum.variants {

        let var_ident = &variant.ident;

        // Does the user have `#[justify = false]` on this variant? => skip_self_just
        // Does the user have `#[justify_inner = false]`? => skip_child_just
        let skip_self_just  = crate::is_justification_disabled_for_variant(variant);
        let skip_child_just = skip_self_just || crate::is_justification_disabled_for_inner_variant(variant);

        match &variant.fields {
            // ---------- Unit Variant ----------
            syn::Fields::Unit => {
                if skip_self_just {
                    // No top-level fields
                    variant_defs.push(quote! { #var_ident, });
                } else {
                    // Insert variant_confidence, variant_justification
                    variant_defs.push(quote! {
                        #var_ident {
                            variant_confidence: f64,
                            variant_justification: String,
                        },
                    });
                }
            }

            // ---------- Named Variant ----------
            syn::Fields::Named(named_fields) => {
                let mut final_fields = Vec::new();

                // If skip_self_just == false => top-level
                if !skip_self_just {
                    final_fields.push(quote! { variant_confidence: f64, });
                    final_fields.push(quote! { variant_justification: String, });
                }

                for field in &named_fields.named {
                    let f_ident = match &field.ident {
                        Some(id) => id,
                        None => continue, // shouldn't happen in Named
                    };

                    let field_ty = &field.ty;

                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let actually_skip   = skip_field_self || skip_child_just;

                    // Always store the original field as is
                    final_fields.push(quote! {
                        #f_ident: #field_ty,
                    });

                    // If not skipping => add `field_confidence`, `field_justification`
                    if !actually_skip {
                        let conf_id = syn::Ident::new(&format!("{}_confidence", f_ident), f_ident.span());
                        let just_id = syn::Ident::new(&format!("{}_justification", f_ident), f_ident.span());
                        final_fields.push(quote! {
                            #conf_id: f64,
                            #just_id: String,
                        });
                    }
                }

                variant_defs.push(quote! {
                    #var_ident {
                        #(#final_fields)*
                    },
                });
            }

            // ---------- Unnamed (tuple) Variant ----------
            syn::Fields::Unnamed(unnamed_fields) => {
                let mut final_fields = Vec::new();

                if !skip_self_just {
                    final_fields.push(quote! { variant_confidence: f64, });
                    final_fields.push(quote! { variant_justification: String, });
                }

                for (i, field) in unnamed_fields.unnamed.iter().enumerate() {
                    let field_ident = syn::Ident::new(&format!("field_{}", i), field.span());

                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let actually_skip = skip_field_self || skip_child_just;

                    let field_ty = &field.ty;

                    // Always store the original field
                    final_fields.push(quote! {
                        #field_ident: #field_ty,
                    });

                    // If not skipping => add `field_i_confidence` and `field_i_justification`
                    if !actually_skip {
                        let conf_id = syn::Ident::new(&format!("field_{}_confidence", i), field.span());
                        let just_id = syn::Ident::new(&format!("field_{}_justification", i), field.span());
                        final_fields.push(quote! {
                            #conf_id: f64,
                            #just_id: String,
                        });
                    }
                }

                variant_defs.push(quote! {
                    #var_ident {
                        #(#final_fields)*
                    },
                });
            }
        }
    }

    // If the user’s original enum had a `#[default]` variant,
    // we can safely do `#[derive(Default)]` on the “WithJustification” enum.
    // Otherwise, we omit Default to avoid parse errors or runtime panics.
    let maybe_default = if user_has_default_variant {
        quote! { #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)] }
    } else {
        quote! { #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] }
    };

    // The flattened enum e.g. `enum MyEnumWithJustification { ... }`
    let enum_with_just_ts = quote! {
        #maybe_default
        pub enum #enum_with_just_ident {
            #(#variant_defs)*
        }
    };

    // The top-level wrapper e.g. 
    //  `pub struct JustifiedMyEnum { variant_confidence: f64, variant_justification: String, variant_selection: MyEnumWithJustification }`
    // always has top-level variant_confidence & variant_justification for the *choice* of variant.
    // That’s how we track which variant was chosen, plus the top-level justification.
    let justified_wrapper_ts = quote! {
        #[derive(Getters, Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[getset(get="pub")]
        pub struct #justified_enum_ident {
            variant_confidence: f64,
            variant_justification: String,
            variant_selection: #enum_with_just_ident,
        }
    };

    quote! {
        #enum_with_just_ts
        #justified_wrapper_ts
    }
}
