crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_enum_with_justification(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span,
    container_docs_str: &str,
) -> proc_macro2::TokenStream {
    trace!("Handling enum justification expansions for '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    // 1) typed expansions
    let (enum_just_ts, enum_conf_ts, justified_enum_ts) =
        generate_enum_justified(ty_ident, data_enum, span);

    // 2) expansions for “to_template_with_justification”
    let template_ts = generate_to_template_with_justification_for_enum(
        ty_ident,
        data_enum,
        container_docs_str
    );

    out.extend(enum_just_ts);
    out.extend(enum_conf_ts);
    out.extend(justified_enum_ts);
    out.extend(template_ts);

    out
}
