// ---------------- [ File: ai-json-template-derive/src/generate_flat_justified_for_named.rs ]
crate::ix!();

/// For each **named struct**, build:
///
/// - A flattened struct `FlatJustifiedFoo { ... }`
/// - An impl `From<FlatJustifiedFoo> for JustifiedFoo`.
///
/// This logic ensures that **leaf types** (`String`, numeric, `HashMap<K,V>`, etc.)
/// do *not* get turned into `StringJustification` or `HashMapConfidence`.
/// Instead, we store a single `String` (justification) and `f32` (confidence) at the parent level.
pub fn generate_flat_justified_for_named(
    ty_ident:     &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span:         proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    let flat_ident       = syn::Ident::new(&format!("FlatJustified{}", ty_ident), span);
    let justified_ident  = syn::Ident::new(&format!("Justified{}",   ty_ident), span);

    // The parent's justification+confidence types, e.g. `FooJustification`, `FooConfidence`.
    let justification_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident    = syn::Ident::new(&format!("{}Confidence",   ty_ident), span);

    let mut flat_fields = Vec::new();
    let mut item_inits  = Vec::new();
    let mut just_inits  = Vec::new();
    let mut conf_inits  = Vec::new();

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => continue, // skip unnamed, or produce error
        };

        // (A) If `#[justify=false]` => skip top-level justification for this field.
        let skip_self_just = is_justification_disabled_for_field(field);

        // (B) If child is a leaf type or has `#[justify_inner=false]`, skip nested flattening.
        // That means we do **not** produce `ChildJustification`; we store a single `String` in the parent's struct.
        let skip_child_just = skip_self_just 
            || is_justification_disabled_for_inner(field)
            || is_leaf_type(&field.ty);

        // 1) Figure out the flattened type for `pub field: ???`
        let flattened_type = match compute_flat_type_for_stamped(&field.ty, skip_child_just, field.span()) {
            Ok(ts) => ts,
            Err(e) => {
                flat_fields.push(e.to_compile_error());
                continue;
            }
        };
        flat_fields.push(quote! {
            #[serde(default)]
            pub #field_ident: #flattened_type,
        });

        // 2) In `impl From<FlatJustifiedFoo> for JustifiedFoo`: do either `... = flat.field`
        //    or `... = From::from(flat.field)`.
        if skip_child_just {
            item_inits.push(quote! {
                #field_ident: flat.#field_ident
            });
        } else {
            // child is presumably a user-defined justification struct => wrap with From
            item_inits.push(quote! {
                #field_ident: ::core::convert::From::from(flat.#field_ident)
            });
        }

        // 3) If `#[justify=false]` is NOT set => we do top-level justification/conf.
        if !skip_self_just {
            let j_id = syn::Ident::new(
                &format!("{}_justification", field_ident),
                field_ident.span()
            );
            let c_id = syn::Ident::new(
                &format!("{}_confidence", field_ident),
                field_ident.span()
            );

            // Add them to the flattened struct:
            flat_fields.push(quote! {
                #[serde(default)]
                pub #j_id: String,
                #[serde(default)]
                pub #c_id: f32,
            });

            // Now decide: if the child is also justification-enabled (skip_child_just == false),
            // we must store them in a nested child struct in the parent's justification.
            if skip_child_just {
                // It's a leaf => parent's justification struct simply has a `String` for justification
                just_inits.push(quote! { #j_id: flat.#j_id });
                conf_inits.push(quote! { #c_id: flat.#c_id });
            } else {
                // It's a user-defined child => parent's justification field is e.g. `ChildJustification`.
                // So we do a small wrapper struct with sub-fields.  Minimal example:
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

    // Flattened struct def:
    let flat_ts = quote! {
        #[derive(
            Default,
            Serialize,
            Deserialize,
            Debug,
            Clone,
            PartialEq
        )]
        pub struct #flat_ident {
            #(#flat_fields)*
        }
    };

    // `impl From<FlatJustifiedFoo> for JustifiedFoo`
    let from_ts = quote! {
        impl From<#flat_ident> for #justified_ident {
            fn from(flat: #flat_ident) -> Self {
                let item = #ty_ident {
                    #(#item_inits, )*
                };
                let justification = #justification_ident {
                    #(#just_inits, )*
                    ..Default::default()
                };
                let confidence = #confidence_ident {
                    #(#conf_inits, )*
                    ..Default::default()
                };
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
