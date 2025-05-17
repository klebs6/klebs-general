// ---------------- [ File: ai-json-template-derive/src/expand_enum_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_enum_with_justification(
    ty_ident:           &syn::Ident,
    data_enum:          &syn::DataEnum,
    span:               proc_macro2::Span,
    container_docs_str: &str
) -> proc_macro2::TokenStream
{
    trace!("expand_enum_with_justification => '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    // 1) Create the flattened enum + Justified wrapper
    let flattened_enum_ts = generate_enum_justified(ty_ident, data_enum, span);
    out.extend(flattened_enum_ts);

    // 2) Add `impl AiJsonTemplateWithJustification` => the schema expansions
    let to_tpl_ts = generate_to_template_with_justification_for_enum(
        ty_ident,
        data_enum,
        container_docs_str,
    );
    out.extend(to_tpl_ts);

    trace!("expand_enum_with_justification => done '{}'", ty_ident);
    out
}
