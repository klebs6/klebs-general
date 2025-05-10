crate::ix!();

/// Generates:
///   - A `FlatJustified[Type]` struct with flattened fields
///   - An `impl From<FlatJustified[Type]> for Justified[Type]`
/// 
/// This includes logic to handle skipping child justification if
/// `#[justify_inner=false]` or if the field is a built-in type.
pub fn generate_flat_justified_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span
) -> (
    proc_macro2::TokenStream, // The FlatJustified struct
    proc_macro2::TokenStream  // The impl From<FlatJustified> for Justified
) {
    let flat_ident = syn::Ident::new(
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

    let mut flat_fields = Vec::new();
    let mut from_field_rebuild_item = Vec::new();
    let mut from_field_rebuild_just = Vec::new();
    let mut from_field_rebuild_conf = Vec::new();

    for field in &named_fields.named {
        let field_ident = field.ident.as_ref().unwrap();
        let field_span = field.span();

        let skip_self_just = is_justification_disabled_for_field(field);
        let skip_child_just = skip_self_just || is_justification_disabled_for_inner(field);

        let field_flat_ty = match compute_flat_type_for_stamped(&field.ty, skip_child_just, field_span) {
            Ok(ts) => ts,
            Err(e) => {
                flat_fields.push(e.to_compile_error());
                continue;
            }
        };

        // flatten the actual data field
        flat_fields.push(quote::quote! {
            #[serde(default)]
            #[getset(get="pub", set="pub")]
            #field_ident: #field_flat_ty,
        });

        // if we do justification:
        if !skip_self_just {
            let just_id = syn::Ident::new(&format!("{}_justification", field_ident), field_span);
            let conf_id = syn::Ident::new(&format!("{}_confidence", field_ident), field_span);

            flat_fields.push(quote::quote! {
                #[serde(default)]
                #[getset(get="pub", set="pub")]
                #just_id: String,

                #[serde(default)]
                #[getset(get="pub", set="pub")]
                #conf_id: f32,
            });

            from_field_rebuild_just.push(quote::quote! {
                .#just_id(flat.#just_id)
            });
            from_field_rebuild_conf.push(quote::quote! {
                .#conf_id(flat.#conf_id)
            });
        }

        // item rebuild logic
        if skip_child_just {
            from_field_rebuild_item.push(quote::quote! {
                #field_ident: flat.#field_ident
            });
        } else {
            from_field_rebuild_item.push(quote::quote! {
                #field_ident: ::std::convert::From::from(flat.#field_ident)
            });
        }
    }

    let flat_ts = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #flat_ident {
            #(#flat_fields)*
        }
    };

    let from_ts = quote::quote! {
        impl ::std::convert::From<#flat_ident> for #justified_ident {
            fn from(flat: #flat_ident) -> Self {
                let item = super::#ty_ident {
                    #(#from_field_rebuild_item),*
                };

                let justification = #justification_ident::builder()
                    #(#from_field_rebuild_just)*
                    .build()
                    .unwrap_or_default();

                let confidence = #confidence_ident::builder()
                    #(#from_field_rebuild_conf)*
                    .build()
                    .unwrap_or_default();

                Self {
                    item,
                    justification,
                    confidence,
                }
            }
        }
    };

    (flat_ts, from_ts)
}
