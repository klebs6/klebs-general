// ---------------- [ File: workspacer-syntax/src/generate_enum_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct EnumSignatureGenerator(ast::Enum);

impl GenerateSignature for ast::Enum {

    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::Enum with opts: {:?}", opts);

        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_enum>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause = full_clean_where_clause(&self.where_clause());

        // If fully_expand, we gather actual variants:
        let variants_text = {
            if let Some(variant_list) = self.variant_list() {
                let items: Vec<String> = variant_list
                    .variants()
                    .map(|v| {
                        let vname = v.name().map(|x| x.text().to_string()).unwrap_or_default();
                        // If it has a field list, we expand them, else just the variant name
                        if let Some(fl) = v.field_list() {
                            match fl {
                                ast::FieldList::RecordFieldList(rfl) => {
                                    let fields: Vec<String> = rfl.fields().map(|f| {
                                        let fname = f
                                            .name()
                                            .map(|n| n.text().to_string())
                                            .unwrap_or_default();
                                        let fty = f
                                            .ty()
                                            .map(|t| t.syntax().text().to_string())
                                            .unwrap_or_default();
                                        format!("{fname}: {fty}")
                                    }).collect();
                                    format!("{vname} {{ {} }}", fields.join(", "))
                                }
                                ast::FieldList::TupleFieldList(tfl) => {
                                    let fields: Vec<String> = tfl.fields().map(|f| {
                                        let fty = f.ty().map(|t| t.syntax().text().to_string()).unwrap_or_default();
                                        fty
                                    }).collect();
                                    format!("{vname}({})", fields.join(", "))
                                }
                            }
                        } else {
                            // no fields
                            vname
                        }
                    })
                    .collect();
                format!("{{\n    {},\n}}", items.join(",\n    "))
            } else {
                ";".to_string()
            }
        };

        let core = format!(
            "{vis_str}enum {name}{generic_params_raw}{where_clause} {variants_text}"
        );

        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}
