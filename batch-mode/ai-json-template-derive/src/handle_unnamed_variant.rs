crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_unnamed_variant(
    var_ident: &syn::Ident,
    unnamed_fields: &syn::FieldsUnnamed,
    skip_self_just: bool,
    is_first_variant: bool
) -> (
    proc_macro2::TokenStream, // variant in Justification enum
    proc_macro2::TokenStream, // variant in Confidence enum
    Vec<String>,              // justification fields (if first variant)
    Vec<String>               // confidence fields (if first variant)
)
{
    debug!(
        "Handling unnamed variant '{}', skip_self_just={}, is_first_variant={}",
        var_ident,
        skip_self_just,
        is_first_variant
    );

    let mut j_fields = Vec::new();
    let mut c_fields = Vec::new();
    let mut out_just_names = Vec::new();
    let mut out_conf_names = Vec::new();

    if !skip_self_just {
        j_fields.push(quote::quote! { variant_justification: String, });
        c_fields.push(quote::quote! { variant_confidence: f32, });

        if is_first_variant {
            out_just_names.push("variant_justification".to_string());
            out_conf_names.push("variant_confidence".to_string());
        }
    }

    for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
        if is_justification_enabled(field) {
            let j_id = syn::Ident::new(
                &format!("field_{}_justification", idx),
                field.span()
            );
            let c_id = syn::Ident::new(
                &format!("field_{}_confidence", idx),
                field.span()
            );
            j_fields.push(quote::quote! { #j_id: String, });
            c_fields.push(quote::quote! { #c_id: f32, });

            if is_first_variant {
                out_just_names.push(format!("field_{}_justification", idx));
                out_conf_names.push(format!("field_{}_confidence", idx));
            }
        }
    }

    let just_variant = quote::quote! {
        #var_ident { #(#j_fields)* }
    };
    let conf_variant = quote::quote! {
        #var_ident { #(#c_fields)* }
    };

    (just_variant, conf_variant, out_just_names, out_conf_names)
}
