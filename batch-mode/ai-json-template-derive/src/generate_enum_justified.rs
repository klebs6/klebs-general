// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> proc_macro2::TokenStream
{
    use quote::quote;

    // e.g. "JustifiedMyEnum"
    let justified_enum_ident = syn::Ident::new(
        &format!("Justified{}", ty_ident),
        span
    );

    let mut variant_defs = Vec::new();

    for variant in &data_enum.variants {
        let var_ident       = &variant.ident;
        let skip_self_just  = crate::is_justification_disabled_for_variant(variant);
        let skip_child_just = skip_self_just || crate::is_justification_disabled_for_inner_variant(variant);

        match &variant.fields {

            // -------------------------------------------
            // (A) Unit Variant
            // -------------------------------------------
            syn::Fields::Unit => {
                if skip_self_just {
                    // For #[justify(false)], the tests want no fields at all in flattened enum
                    // => Just `X,`
                    variant_defs.push(quote! {
                        #var_ident,
                    });
                } else {
                    // Otherwise we define { variant_confidence: f64, variant_justification: String }
                    variant_defs.push(quote! {
                        #var_ident {
                            variant_confidence: f64,
                            variant_justification: String
                        },
                    });
                }
            }

            // -------------------------------------------
            // (B) Named fields variant
            // -------------------------------------------
            syn::Fields::Named(named) => {
                let mut final_fields = Vec::new();
                // If skip_self_just == false, add top-level variant_confidence/justification
                if !skip_self_just {
                    final_fields.push(quote!{ variant_confidence: f64 });
                    final_fields.push(quote!{ variant_justification: String });
                }

                for field in &named.named {
                    let field_ident = field.ident.as_ref().unwrap();
                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let actually_skip   = skip_field_self || skip_child_just;
                    let justified_ty    = crate::justified_type(&field.ty);

                    // Always store the main field
                    final_fields.push(quote! {
                        #field_ident: #justified_ty
                    });

                    // If not skipping => also add field_confidence + field_justification
                    if !actually_skip {
                        let conf_id = syn::Ident::new(
                            &format!("{}_confidence", field_ident),
                            field_ident.span()
                        );
                        let just_id = syn::Ident::new(
                            &format!("{}_justification", field_ident),
                            field_ident.span()
                        );
                        final_fields.push(quote! {
                            #conf_id: f64
                        });
                        final_fields.push(quote! {
                            #just_id: String
                        });
                    }
                }

                variant_defs.push(quote! {
                    #var_ident {
                        #( #final_fields ),*
                    },
                });
            }

            // -------------------------------------------
            // (C) Unnamed / tuple variant
            // -------------------------------------------
            syn::Fields::Unnamed(unnamed) => {
                let mut final_fields = Vec::new();

                if !skip_self_just {
                    final_fields.push(quote!{ variant_confidence: f64 });
                    final_fields.push(quote!{ variant_justification: String });
                }

                for (i, field) in unnamed.unnamed.iter().enumerate() {
                    let skip_field_self = crate::is_justification_disabled_for_field(field);
                    let actually_skip   = skip_field_self || skip_child_just;
                    let justified_ty    = crate::justified_type(&field.ty);
                    let field_ident     = syn::Ident::new(&format!("field_{}", i), field.span());

                    // Always store the main field
                    final_fields.push(quote! {
                        #field_ident: #justified_ty
                    });

                    if !actually_skip {
                        let conf_id = syn::Ident::new(&format!("field_{}_confidence", i), field.span());
                        let just_id = syn::Ident::new(&format!("field_{}_justification", i), field.span());
                        final_fields.push(quote! {
                            #conf_id: f64
                        });
                        final_fields.push(quote! {
                            #just_id: String
                        });
                    }
                }

                variant_defs.push(quote! {
                    #var_ident {
                        #( #final_fields ),*
                    },
                });
            }
        }
    }

    quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum #justified_enum_ident {
            #( #variant_defs )*
        }
    }
}
