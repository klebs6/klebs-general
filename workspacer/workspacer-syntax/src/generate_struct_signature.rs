// ---------------- [ File: src/generate_struct_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct StructSignatureGenerator(ast::Struct);

impl GenerateSignature for ast::Struct {
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::Struct with opts: {:?}", opts);

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

        let name_str = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_struct>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        // If fully_expand = true, let's gather actual fields:
        // If false, placeholders.
        let fields_text = if *opts.fully_expand() {
            if let Some(fl) = self.field_list() {
                match fl {
                    ast::FieldList::RecordFieldList(rfl) => {
                        let all_fields: Vec<String> = rfl
                            .fields()
                            .map(|field| {
                                let fname = field
                                    .name()
                                    .map(|n| n.text().to_string())
                                    .unwrap_or_default();
                                let fty = field
                                    .ty()
                                    .map(|t| t.syntax().text().to_string())
                                    .unwrap_or_default();
                                format!("    {}: {},", fname, fty)
                            })
                            .collect();
                        format!("{{\n{}\n}}", all_fields.join("\n"))
                    }
                    ast::FieldList::TupleFieldList(tfl) => {
                        let all_fields: Vec<String> = tfl
                            .fields()
                            .map(|field| {
                                let vis = field
                                    .visibility()
                                    .map(|v| format!("{} ", v.syntax().text()))
                                    .unwrap_or_default();
                                let fty = field
                                    .ty()
                                    .map(|t| t.syntax().text().to_string())
                                    .unwrap_or_default();
                                format!("    {}{},", vis, fty)
                            })
                            .collect();
                        format!("(\n{}\n);", all_fields.join("\n"))
                    }
                }
            } else {
                // no fields => e.g. `struct Foo;`
                ";".to_string()
            }
        } else {
            // minimal or placeholder approach
            "{ /* fields omitted */ }".to_string()
        };

        let core = format!(
            "{vis_str}struct {name_str}{generic_params_raw}{where_clause} {fields_text}"
        );

        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}
