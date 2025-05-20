// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream {
    use quote::quote;

    // e.g. "JustifiedGeometry2d"
    let justified_enum_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

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
                    let field_ident = match &field.ident {
                        Some(id) => id,
                        None => continue, // shouldn't happen in Named
                    };

                    let justified_ty = justified_type(&field.ty);

                    // 1) Gather all attributes from the original field
                    let original_attrs = &field.attrs;

                    // 2) Filter only the ones that start with `#[serde(...)]`
                    let serde_attrs: Vec<_> = original_attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("serde"))
                        .collect();

                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let actually_skip   = skip_field_self || skip_child_just;

                    // Always store the original field as is
                    final_fields.push(quote! {
                        #( #serde_attrs )*
                        #field_ident: #justified_ty,
                    });

                    // If not skipping => add `field_confidence`, `field_justification`
                    if !actually_skip {
                        let conf_id = syn::Ident::new(&format!("{}_confidence", field_ident), field_ident.span());
                        let just_id = syn::Ident::new(&format!("{}_justification", field_ident), field_ident.span());
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

                    let justified_ty = justified_type(&field.ty);

                    // Always store the original field
                    final_fields.push(quote! {
                        #field_ident: #justified_ty,
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

    let enum_def = quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum #justified_enum_ident {
            #(#variant_defs)*
        }
    };

    quote!{
        #enum_def
    }
}
