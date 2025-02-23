crate::ix!();

// --------------------------------------------------------------------
// Implementation for `ast::Enum`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Enum {
    fn generate_signature(&self) -> String {

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

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        // optionally gather enum variants, but for brevity we skip
        let core = format!(
            "{vis_str}enum {name}{generic_params_raw}{where_clause} ",
        );
        let final_sig = format!("{core}");
        post_process_spacing(&final_sig)
    }
}
