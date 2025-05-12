// ---------------- [ File: ai-json-template-derive/src/handle_named_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_named_variant(
    var_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    skip_self_just: bool,
    is_first_variant: bool
) -> (
    proc_macro2::TokenStream, // variant in the Justification enum
    proc_macro2::TokenStream, // variant in the Confidence enum
    Vec<String>,              // new justification field names
    Vec<String>               // new confidence field names
)
{
    debug!(
        "Handling named variant '{}', skip_self_just={}, is_first_variant={}",
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

    for field in &named_fields.named {
        if is_justification_enabled(field) {
            let f_id = field.ident.as_ref().unwrap();
            let j_id = syn::Ident::new(
                &format!("{}_justification", f_id),
                f_id.span()
            );
            let c_id = syn::Ident::new(
                &format!("{}_confidence", f_id),
                f_id.span()
            );
            j_fields.push(quote::quote! { #j_id: String, });
            c_fields.push(quote::quote! { #c_id: f32, });

            if is_first_variant {
                out_just_names.push(format!("{}_justification", f_id));
                out_conf_names.push(format!("{}_confidence", f_id));
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
