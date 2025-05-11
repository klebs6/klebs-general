crate::ix!();

/// A little helper that collects expansions for one named field.
/// Returns:
///   - `flattened_decls`: the lines we inject into the Flat enum variant (e.g. `#[serde(default)] name: SomeType,`).
///   - `item_init`: how to initialize `item.field` in the final `From<...>` arm.
///   - `just_init`: how to initialize `justification.field_justification`.
///   - `conf_init`: how to initialize `confidence.field_confidence`.
///
/// If `skip_self_just == true`, we skip top-level justification/conf for this field.
/// If `parent_skip_child == true`, we do **not** flatten the child type, so we just do a direct assignment.
pub fn flatten_named_field(
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    skip_self_just: bool,
    parent_skip_child: bool,
) -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    use quote::quote;

    let mut flattened_decls = Vec::new();

    // Compute the flattened type (leaf → same type, or nested → FlattenedChild)
    let flattened_type = match crate::compute_flat_type_for_stamped(field_ty, parent_skip_child, field_ty.span()) {
        Ok(ts) => ts,
        Err(e) => {
            // Return a compile_error in the variant declarations
            return (vec![e.to_compile_error()], quote!(), quote!(), quote!());
        }
    };

    // 1) Declare the flattened field, with no `pub` (avoid E0449)
    flattened_decls.push(quote! {
        #[serde(default)]
        #field_ident: #flattened_type,
    });

    // 2) For the final `item` constructor, do either `field_ident: flat.field_ident`
    //    or `From::from(flat.field_ident)`
    let item_init = if parent_skip_child {
        quote! { #field_ident: #field_ident }
    } else {
        quote! { #field_ident: ::core::convert::From::from(#field_ident) }
    };

    // 3) If `skip_self_just == false`, add top-level `fieldname_justification: String` + `fieldname_confidence: f32`
    if !skip_self_just {
        let j_id = syn::Ident::new(&format!("{}_justification", field_ident), field_ident.span());
        let c_id = syn::Ident::new(&format!("{}_confidence",   field_ident), field_ident.span());

        // Add them to the flattened variant
        flattened_decls.push(quote! {
            #[serde(default)]
            #j_id: String,
            #[serde(default)]
            #c_id: f32,
        });

        // For the final justification/conf, if the child is non-leaf, embed it, else direct
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
        // No justification/conf fields
        (flattened_decls, item_init, quote!(), quote!())
    }
}
