// ---------------- [ File: src/generate_macro_signature.rs ]
crate::ix!();

// --------------------------------------------------------------------
// Implementation for `ast::MacroRules`
// --------------------------------------------------------------------
impl GenerateSignature for ast::MacroRules {
    fn generate_signature(&self) -> String {

        let name = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_macro>".to_string());

        let core = format!("macro_rules! {name} ");
        let final_sig = format!("{core}");
        post_process_spacing(&final_sig)
    }
}
