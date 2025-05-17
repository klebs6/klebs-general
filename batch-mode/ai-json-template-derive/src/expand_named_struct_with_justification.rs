// ---------------- [ File: ai-json-template-derive/src/expand_named_struct_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_named_struct_with_justification(
    ty_ident:           &syn::Ident,
    ds:                 &syn::DataStruct,
    span:               proc_macro2::Span,
    container_docs_str: &str
) -> proc_macro2::TokenStream
{
    trace!("expand_named_struct_with_justification => '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    match &ds.fields {
        syn::Fields::Named(named_fields) => {
            // 1) Build the single flattened “JustifiedFoo” struct
            let flattened_ts = generate_justified_structs_for_named(ty_ident, named_fields, span);
            out.extend(flattened_ts);

            // 2) Generate the `impl AiJsonTemplateWithJustification` that
            //    includes `to_template_with_justification()`
            let to_tpl_ts = generate_to_template_with_justification_for_named(
                ty_ident,
                named_fields,
                container_docs_str
            );
            out.extend(to_tpl_ts);
        }
        _ => {
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification only supports named structs"
            );
            out.extend(e.to_compile_error());
        }
    }

    trace!("expand_named_struct_with_justification => done '{}'", ty_ident);
    out
}

