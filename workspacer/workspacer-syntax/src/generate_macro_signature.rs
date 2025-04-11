// ---------------- [ File: workspacer-syntax/src/generate_macro_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct MacroRulesSignatureGenerator(ast::MacroRules);

impl GenerateSignature for ast::MacroRules {
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::MacroRules with opts: {:?}", opts);

        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let name = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_macro>".to_string());

        let body_text = if *opts.fully_expand() {
            trace!("Fully expanding macro body");
            if let Some(tt) = self.token_tree() {
                let body_str = tt.syntax().text().to_string();
                debug!("Macro body content: {}", body_str);
                format!("{{ {body_str} }}")
            } else {
                warn!("No macro token tree found");
                "{ /* macro body not available */ }".to_string()
            }
        } else {
            "{ /* macro body omitted */ }".to_string()
        };

        let core = format!("macro_rules! {name} {body_text}");

        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}
