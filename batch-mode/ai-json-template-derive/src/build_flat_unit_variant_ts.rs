crate::ix!();

/// Constructs the snippet for the flattened variant itself. If `skip_self_just` is true,
/// it has no justification/confidence fields; otherwise, it has them.
pub fn build_flat_unit_variant_ts(
    skip_self_just:    bool,
    renamed_var_ident: &syn::Ident
) -> proc_macro2::TokenStream {
    trace!(
        "build_flat_unit_variant_ts: skip_self_just={}, variant='{}'",
        skip_self_just,
        renamed_var_ident
    );

    if skip_self_just {
        quote::quote! {
            #renamed_var_ident,
        }
    } else {
        quote::quote! {
            #renamed_var_ident {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32
            },
        }
    }
}
