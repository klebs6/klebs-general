// ---------------- [ File: ai-json-template-derive/src/expand_ai_json_template_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_ai_json_template_with_justification(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    trace!("Entering expand_ai_json_template_with_justification for '{}'", ast.ident);

    let span = ast.span();
    let ty_ident = &ast.ident;
    let doc_lines = gather_doc_comments(&ast.attrs);
    let container_docs_str = doc_lines.join("\n");

    let mut out = proc_macro2::TokenStream::new();

    match &ast.data {
        syn::Data::Struct(ds) => {
            trace!("Struct detected => dispatching to expand_named_struct_with_justification if fields are named");
            let tokens = expand_named_struct_with_justification(
                ty_ident,
                ds,
                span,
                &container_docs_str,
            );
            out.extend(tokens);
        }
        syn::Data::Enum(data_enum) => {
            trace!("Enum detected => dispatching to expand_enum_with_justification");
            let tokens = expand_enum_with_justification(
                ty_ident,
                data_enum,
                span,
                &container_docs_str,
            );
            out.extend(tokens);
        }
        syn::Data::Union(_) => {
            trace!("Union detected => not supported by AiJsonTemplateWithJustification");
            let err = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification not supported on unions."
            );
            out.extend(err.to_compile_error());
        }
    }

    trace!("Exiting expand_ai_json_template_with_justification for '{}'", ast.ident);
    out
}
