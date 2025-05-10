crate::ix!();

/// This subroutine orchestrates the entire process of deriving
/// `AiJsonTemplateWithJustification` for named structs *or* enums.
///
/// It delegates to separate helpers for:
///  - Generating the normal `JustifiedX` struct items
///  - Generating the `FlatJustifiedX` struct or enum items
///  - Generating `impl From<FlatJustifiedX> for JustifiedX`
///  - Generating the `to_template_with_justification()` expansions
///
/// This function is called from `derive_ai_json_template_with_justification`
/// after we parse the main AST.
pub fn expand_ai_json_template_with_justification(
    ast: &syn::DeriveInput
) -> proc_macro2::TokenStream {
    let span = ast.span();
    let ty_ident = &ast.ident;
    let container_docs_vec = gather_doc_comments(&ast.attrs);
    let container_docs_str = container_docs_vec.join("\n");

    let mut output_ts = proc_macro2::TokenStream::new();

    match &ast.data {
        syn::Data::Struct(ds) => {
            match &ds.fields {
                syn::Fields::Named(named_fields) => {
                    // 1) Generate normal Justified structs
                    let (just_ts, conf_ts, justified_ts, accessor_ts) =
                        generate_justified_structs_for_named(ty_ident, named_fields, span);

                    // 2) Generate the `FlatJustifiedX` + `From<FlatJustifiedX> for JustifiedX`
                    let (flat_ts, from_ts) =
                        generate_flat_justified_for_named(ty_ident, named_fields, span);

                    // 3) Generate the `to_template_with_justification` expansions
                    let tpls = generate_to_template_with_justification_for_named(
                        ty_ident,
                        named_fields,
                        &container_docs_str
                    );

                    output_ts.extend(just_ts);
                    output_ts.extend(conf_ts);
                    output_ts.extend(justified_ts);
                    output_ts.extend(accessor_ts);
                    output_ts.extend(flat_ts);
                    output_ts.extend(from_ts);
                    output_ts.extend(tpls);
                }
                _ => {
                    let e = syn::Error::new(
                        span,
                        "AiJsonTemplateWithJustification only supports named fields for structs."
                    );
                    output_ts.extend(e.to_compile_error());
                }
            }
        }
        syn::Data::Enum(data_enum) => {
            // 1) Generate minimal Justification + Confidence + Justified for the enum
            let (enum_just, enum_conf, justified_enum) =
                generate_enum_justified(ty_ident, span);

            // 2) Generate expansions for `to_template_with_justification()`
            let template_expansions = generate_to_template_with_justification_for_enum(
                ty_ident,
                data_enum,
                &container_docs_str
            );

            // 3) Generate a FlatJustifiedEnum + impl From<...> for JustifiedEnum
            let (flat_enum_ts, from_enum_ts) =
                generate_flat_justified_for_enum(ty_ident, data_enum, span);

            output_ts.extend(enum_just);
            output_ts.extend(enum_conf);
            output_ts.extend(justified_enum);
            output_ts.extend(template_expansions);
            output_ts.extend(flat_enum_ts);
            output_ts.extend(from_enum_ts);
        }
        syn::Data::Union(_) => {
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification not supported on unions."
            );
            output_ts.extend(e.to_compile_error());
        }
    }

    output_ts
}

