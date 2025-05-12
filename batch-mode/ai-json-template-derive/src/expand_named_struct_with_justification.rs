crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_named_struct_with_justification(
    ty_ident: &syn::Ident,
    ds: &syn::DataStruct,
    span: proc_macro2::Span,
    container_docs_str: &str,
) -> proc_macro2::TokenStream {
    trace!("Handling named struct justification expansions for '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();
    match &ds.fields {
        syn::Fields::Named(named_fields) => {
            let (just_ts, conf_ts, justified_ts, accessor_ts) =
                generate_justified_structs_for_named(ty_ident, named_fields, span);

            let tpls = generate_to_template_with_justification_for_named(
                ty_ident,
                named_fields,
                container_docs_str
            );

            out.extend(just_ts);
            out.extend(conf_ts);
            out.extend(justified_ts);
            out.extend(accessor_ts);
            out.extend(tpls);
        }
        _ => {
            warn!("Struct is not named => returning error");
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification only supports named structs"
            );
            out.extend(e.to_compile_error());
        }
    }
    out
}
