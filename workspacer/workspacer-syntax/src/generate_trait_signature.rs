crate::ix!();

// --------------------------------------------------------------------
// Implementation for `ast::Trait`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Trait {
    fn generate_signature(&self) -> String {

        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_trait>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
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

        let core = format!(
            "{vis_str}trait {name}{generic_params_raw}{where_clause} ",
        );
        let final_sig = format!("{core}");
        post_process_spacing(&final_sig)
    }
}
