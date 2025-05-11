// ---------------- [ File: ai-json-template-derive/src/flatten_unnamed_field.rs ]
crate::ix!();

/// Same as `flatten_named_field` but for a tuple (unnamed) field, e.g. `field_0`.
pub fn flatten_unnamed_field(
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    skip_self_just: bool,
    parent_skip_child: bool,
) -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    use quote::quote;

    let mut flattened_decls = Vec::new();

    // Flatten the type
    let flattened_type = match crate::compute_flat_type_for_stamped(field_ty, parent_skip_child, field_ty.span()) {
        Ok(ts) => ts,
        Err(e) => {
            return (vec![e.to_compile_error()], quote!(), quote!(), quote!());
        }
    };

    // 1) No `pub`, to avoid E0449
    flattened_decls.push(quote! {
        #[serde(default)]
        #field_ident: #flattened_type,
    });

    // 2) item init
    let item_init = if parent_skip_child {
        quote! { #field_ident }
    } else {
        quote! { ::core::convert::From::from(#field_ident) }
    };

    // 3) optional justification/conf
    if !skip_self_just {
        let j_id = syn::Ident::new(
            &format!("{}_justification", field_ident),
            field_ident.span()
        );
        let c_id = syn::Ident::new(
            &format!("{}_confidence", field_ident),
            field_ident.span()
        );

        flattened_decls.push(quote! {
            #[serde(default)]
            #j_id: String,
            #[serde(default)]
            #c_id: f32,
        });

        let just_init = if parent_skip_child {
            quote! { #j_id: #j_id }
        } else {
            let child_just = crate::child_ty_to_just(field_ty);
            quote! {
                #j_id: #child_just {
                    detail_justification: #j_id,
                    ..::core::default::Default::default()
                }
            }
        };
        let conf_init = if parent_skip_child {
            quote! { #c_id: #c_id }
        } else {
            let child_conf = crate::child_ty_to_conf(field_ty);
            quote! {
                #c_id: #child_conf {
                    detail_confidence: #c_id,
                    ..::core::default::Default::default()
                }
            }
        };

        (flattened_decls, item_init, just_init, conf_init)
    } else {
        (flattened_decls, item_init, quote!(), quote!())
    }
}
