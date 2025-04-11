crate::ix!();

#[derive(Debug, Clone)]
pub struct MacroCallSignatureGenerator(ast::MacroCall);

impl GenerateSignature for ast::MacroCall {
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::MacroCall with opts: {:?}", opts);

        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let path_str = self
            .path()
            .map(|p| p.syntax().text().to_string())
            .unwrap_or_else(|| {
                warn!("Macro path not found for MacroCall");
                "<unknown_macro>".to_string()
            });
        debug!("MacroCall path: {}", path_str);

        let call_body = if *opts.fully_expand() {
            trace!("Fully expanding macro call body");
            if let Some(tt) = self.token_tree() {
                let body_str = tt.syntax().text().to_string();
                debug!("MacroCall token_tree: {}", body_str);
                body_str
            } else {
                warn!("No token_tree found for MacroCall");
                "{ /* empty */ }".to_string()
            }
        } else {
            "{ /* ... */ }".to_string()
        };

        let core = format!("{path_str}!{call_body}");
        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}
