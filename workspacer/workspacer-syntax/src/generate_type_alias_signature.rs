// ---------------- [ File: src/generate_type_alias_signature.rs ]
crate::ix!();

// --------------------------------------------------------------------
// Implementation for `ast::TypeAlias`
// --------------------------------------------------------------------
impl GenerateSignature for ast::TypeAlias {
    fn generate_signature(&self) -> String {

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_type_alias>".to_string());

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

        // Get the aliased type
        let aliased_type = self
            .ty()
            .map(|ty| ty.syntax().text().to_string())
            .unwrap_or_else(|| "<unknown_aliased_type>".to_string());

        // Possibly `pub `
        let visibility = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let core = format!(
            "{visibility}type {name}{generic_params_raw}{where_clause} = {aliased_type};",
        );

        let final_sig = format!("{core}");
        post_process_spacing(&final_sig)
    }
}
