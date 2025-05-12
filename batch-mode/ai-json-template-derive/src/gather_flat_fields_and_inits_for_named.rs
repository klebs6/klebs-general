// ---------------- [ File: ai-json-template-derive/src/gather_flat_fields_and_inits_for_named.rs ]
crate::ix!();

pub fn gather_flat_fields_and_inits_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    flat_fields: &mut Vec<proc_macro2::TokenStream>,
    item_inits: &mut Vec<proc_macro2::TokenStream>,
    just_inits: &mut Vec<proc_macro2::TokenStream>,
    conf_inits: &mut Vec<proc_macro2::TokenStream>,
) {
    trace!("gather_flat_fields_and_inits_for_named: starting for '{}'", ty_ident);

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                warn!("Encountered unnamed field in a named struct; skipping");
                continue;
            }
        };

        let skip_self_just = is_justification_disabled_for_field(field);
        let skip_child_just =
            skip_self_just || is_justification_disabled_for_inner(field) || is_leaf_type(&field.ty);

        debug!(
            "Field '{}' => skip_self_just={} skip_child_just={}",
            field_ident, skip_self_just, skip_child_just
        );

        match compute_flat_type_for_stamped(&field.ty, skip_child_just, field.span()) {
            Ok(flattened_type) => {
                flat_fields.push(quote! {
                    #[serde(default)]
                    pub #field_ident: #flattened_type,
                });
            }
            Err(e) => {
                error!("Error flattening field '{}': {:?}", field_ident, e);
                flat_fields.push(e.to_compile_error());
                continue;
            }
        }

        // item_inits => direct or From::from
        if skip_child_just {
            item_inits.push(quote! {
                #field_ident: flat.#field_ident
            });
        } else {
            item_inits.push(quote! {
                #field_ident: ::core::convert::From::from(flat.#field_ident)
            });
        }

        // top-level justification/conf if not skip_self_just
        if !skip_self_just {
            let j_id = syn::Ident::new(
                &format!("{}_justification", field_ident),
                field_ident.span()
            );
            let c_id = syn::Ident::new(
                &format!("{}_confidence", field_ident),
                field_ident.span()
            );

            flat_fields.push(quote! {
                #[serde(default)]
                pub #j_id: String,
                #[serde(default)]
                pub #c_id: f32,
            });

            if skip_child_just {
                just_inits.push(quote! { #j_id: flat.#j_id });
                conf_inits.push(quote! { #c_id: flat.#c_id });
            } else {
                let child_just_ty = child_ty_to_just(&field.ty);
                let child_conf_ty = child_ty_to_conf(&field.ty);

                just_inits.push(quote! {
                    #j_id: #child_just_ty {
                        detail_justification: flat.#j_id,
                        ..::core::default::Default::default()
                    }
                });
                conf_inits.push(quote! {
                    #c_id: #child_conf_ty {
                        detail_confidence: flat.#c_id,
                        ..::core::default::Default::default()
                    }
                });
            }
        }
    }

    trace!("gather_flat_fields_and_inits_for_named: done collecting fields and inits.");
}
