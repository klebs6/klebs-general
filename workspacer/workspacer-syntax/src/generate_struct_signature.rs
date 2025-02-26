// ---------------- [ File: src/generate_struct_signature.rs ]
crate::ix!();

// --------------------------------------------------------------------
// Implementation for `ast::Struct`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Struct {
    fn generate_signature(&self) -> String {

        // Possibly pub
        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
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

        // optional: gather fields for display
        let fields_text = if let Some(fl) = self.field_list() {
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
            // fallback: no fields => e.g. "struct Foo;"
            "{ /* ... */ }".to_string()
        };

        let core = format!(
            "{vis_str}struct {name}{generic_params_raw}{where_clause} {fields_text}",
        );
        let final_sig = format!("{core}");
        post_process_spacing(&final_sig)
    }
}
