// ---------------- [ File: workspacer-syntax/src/generate_type_alias_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct TypeAliasSignatureGenerator(ast::TypeAlias);

impl GenerateSignature for ast::TypeAlias {
    fn generate_signature(&self) -> String {
        self.generate_signature_with_opts(&SignatureOptions::default())
    }

    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::TypeAlias with opts: {:?}", opts);

        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_type_alias>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause = full_clean_where_clause(&self.where_clause());

        let visibility = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        // Always show the real aliased type, ignoring .fully_expand().
        // If there's no actual type node, we show "<unknown_aliased_type>" instead.
        let aliased_type = self
            .ty()
            .map(|ty| ty.syntax().text().to_string())
            .unwrap_or_else(|| "<unknown_aliased_type>".to_string());

        let core = format!(
            "{visibility}type {name}{generic_params_raw}{where_clause} = {aliased_type};"
        );

        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}
