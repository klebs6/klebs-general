// ---------------- [ File: ai-json-template-derive/src/expand_ai_json_template_with_justification.rs ]
crate::ix!();

/// The main subroutine that orchestrates “AiJsonTemplateWithJustification” expansions:
pub fn expand_ai_json_template_with_justification(
    ast: &syn::DeriveInput
) -> proc_macro2::TokenStream {
    let span = ast.span();
    let ty_ident = &ast.ident;

    // Gather container-level doc comments
    let doc_lines = gather_doc_comments(&ast.attrs);
    let container_docs_str = doc_lines.join("\n");

    let mut out = proc_macro2::TokenStream::new();

    match &ast.data {
        // ------------------------------------------------------------
        // Named Struct => produce FooJustification, FooConfidence, JustifiedFoo
        // plus the named approach for each field => typical approach
        // ------------------------------------------------------------
        syn::Data::Struct(ds) => {
            match &ds.fields {
                syn::Fields::Named(named_fields) => {
                    let (just_ts, conf_ts, justified_ts, accessor_ts)
                        = generate_justified_structs_for_named(ty_ident, named_fields, span);

                    let tpls = generate_to_template_with_justification_for_named(
                        ty_ident,
                        named_fields,
                        &container_docs_str
                    );
                    out.extend(just_ts);
                    out.extend(conf_ts);
                    out.extend(justified_ts);
                    out.extend(accessor_ts);
                    out.extend(tpls);
                },
                _ => {
                    let e = syn::Error::new(span, "AiJsonTemplateWithJustification only supports named structs");
                    out.extend(e.to_compile_error());
                }
            }
        }

        // ------------------------------------------------------------
        // Enum => produce typed justification/confidence for each variant
        // plus the normal expansions for to_template_with_justification
        // ------------------------------------------------------------
        syn::Data::Enum(data_enum) => {
            // 1) typed expansions
            let (enum_just_ts, enum_conf_ts, justified_enum_ts) 
                = generate_enum_justified(ty_ident, data_enum, span);

            // 2) expansions for the “to_template_with_justification” method 
            //    that injects `field_justification` placeholders, etc.
            let template_ts = generate_to_template_with_justification_for_enum(
                ty_ident,
                data_enum,
                &container_docs_str
            );

            out.extend(enum_just_ts);
            out.extend(enum_conf_ts);
            out.extend(justified_enum_ts);
            out.extend(template_ts);
        }

        syn::Data::Union(_) => {
            let e = syn::Error::new(span, "AiJsonTemplateWithJustification not supported on unions.");
            out.extend(e.to_compile_error());
        }
    }

    out
}
