// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

/// Builds the typed justification and confidence enums, plus a JustifiedEnum struct,
/// ensuring that each variant only declares the fields that variant actually needs.
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> (
    proc_macro2::TokenStream, // enum FooJustification
    proc_macro2::TokenStream, // enum FooConfidence
    proc_macro2::TokenStream, // struct JustifiedFoo
)
{
    use quote::quote;

    let enum_just_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let enum_conf_ident = syn::Ident::new(&format!("{}Confidence",   ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}",    ty_ident), span);

    // For the default() impl, we track the *first* variant’s fields
    let mut first_variant_ident       = None;
    let mut first_variant_just_fields = Vec::<String>::new();
    let mut first_variant_conf_fields = Vec::<String>::new();

    let mut just_variants = Vec::new();
    let mut conf_variants = Vec::new();

    for (i, variant) in data_enum.variants.iter().enumerate() {

        let var_ident = &variant.ident;

        if i == 0 {
            first_variant_ident = Some(var_ident.clone());
        }

        let skip_self_just = crate::is_justification_disabled_for_variant(variant);

        match &variant.fields {
            // ----------------- Unit variant -----------------
            syn::Fields::Unit => {
                if skip_self_just {
                    just_variants.push(quote! { #var_ident {} });
                    conf_variants.push(quote! { #var_ident {} });
                } else {
                    just_variants.push(quote! {
                        #var_ident { variant_justification: String }
                    });
                    conf_variants.push(quote! {
                        #var_ident { variant_confidence: f32 }
                    });
                    if i == 0 {
                        first_variant_just_fields.push("variant_justification".to_string());
                        first_variant_conf_fields.push("variant_confidence".to_string());
                    }
                }
            }

            // ----------------- Named variant -----------------
            syn::Fields::Named(named_fields) => {
                let mut j_fields = Vec::new();
                let mut c_fields = Vec::new();

                // top-level variant_just/conf
                if !skip_self_just {
                    j_fields.push(quote! { variant_justification: String, });
                    c_fields.push(quote! { variant_confidence: f32, });
                    if i == 0 {
                        first_variant_just_fields.push("variant_justification".to_string());
                        first_variant_conf_fields.push("variant_confidence".to_string());
                    }
                }

                // For each named field that has justification
                for field in &named_fields.named {
                    if crate::is_justification_enabled(field) {
                        let f_id = field.ident.as_ref().unwrap();
                        let j_id = syn::Ident::new(
                            &format!("{}_justification", f_id),
                            f_id.span()
                        );
                        let c_id = syn::Ident::new(
                            &format!("{}_confidence", f_id),
                            f_id.span()
                        );
                        j_fields.push(quote! { #j_id: String, });
                        c_fields.push(quote! { #c_id: f32, });
                        if i == 0 {
                            first_variant_just_fields.push(format!("{}_justification", f_id));
                            first_variant_conf_fields.push(format!("{}_confidence", f_id));
                        }
                    }
                }

                just_variants.push(quote! {
                    #var_ident { #(#j_fields)* }
                });
                conf_variants.push(quote! {
                    #var_ident { #(#c_fields)* }
                });
            }

            // ----------------- Unnamed (tuple) variant -----------------
            syn::Fields::Unnamed(unnamed_fields) => {
                let mut j_fields = Vec::new();
                let mut c_fields = Vec::new();

                if !skip_self_just {
                    j_fields.push(quote! { variant_justification: String, });
                    c_fields.push(quote! { variant_confidence: f32, });
                    if i == 0 {
                        first_variant_just_fields.push("variant_justification".to_string());
                        first_variant_conf_fields.push("variant_confidence".to_string());
                    }
                }

                for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
                    if crate::is_justification_enabled(field) {
                        let j_id = syn::Ident::new(
                            &format!("field_{}_justification", idx),
                            field.span()
                        );
                        let c_id = syn::Ident::new(
                            &format!("field_{}_confidence", idx),
                            field.span()
                        );
                        j_fields.push(quote! { #j_id: String, });
                        c_fields.push(quote! { #c_id: f32, });
                        if i == 0 {
                            first_variant_just_fields.push(format!("field_{}_justification", idx));
                            first_variant_conf_fields.push(format!("field_{}_confidence", idx));
                        }
                    }
                }

                just_variants.push(quote! {
                    #var_ident { #(#j_fields)* }
                });
                conf_variants.push(quote! {
                    #var_ident { #(#c_fields)* }
                });
            }
        }
    }

    // --- Build the final Justification enum ---
    let enum_just_ts = {
        let variants_ts = quote! { #( #just_variants ),* };
        let default_impl = if let Some(ref first_var) = first_variant_ident {
            let init_fields: Vec<_> = first_variant_just_fields.iter().map(|f_str| {
                let f_id = syn::Ident::new(f_str, span);
                quote! { #f_id: ::core::default::Default::default() }
            }).collect();
            quote! {
                impl ::core::default::Default for #enum_just_ident {
                    fn default() -> Self {
                        #enum_just_ident::#first_var { #( #init_fields ),* }
                    }
                }
            }
        } else {
            quote!()
        };

        quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            enum #enum_just_ident {
                #variants_ts
            }
            #default_impl
        }
    };

    // --- Build the final Confidence enum ---
    let enum_conf_ts = {
        let variants_ts = quote! { #( #conf_variants ),* };
        let default_impl = if let Some(ref first_var) = first_variant_ident {
            let init_fields: Vec<_> = first_variant_conf_fields.iter().map(|f_str| {
                let f_id = syn::Ident::new(f_str, span);
                quote! { #f_id: ::core::default::Default::default() }
            }).collect();
            quote! {
                impl ::core::default::Default for #enum_conf_ident {
                    fn default() -> Self {
                        #enum_conf_ident::#first_var { #( #init_fields ),* }
                    }
                }
            }
        } else {
            quote!()
        };

        quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            enum #enum_conf_ident {
                #variants_ts
            }
            #default_impl
        }
    };

    // --- The JustifiedFoo struct ---
    let justified_ts = quote! {

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Getters, Setters)]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            item:          #ty_ident,
            justification: #enum_just_ident,
            confidence:    #enum_conf_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };

    (enum_just_ts, enum_conf_ts, justified_ts)
}
