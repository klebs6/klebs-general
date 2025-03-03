// ---------------- [ File: src/generate_function_signature.rs ]
crate::ix!();

/// Minimal post-processing to ensure correct spacing around arrows, where clauses, etc.
pub fn post_process_spacing(signature: &str) -> String {
    signature
        .replace(")->", ") ->")
        .replace(">where", "> where")
}

#[derive(Debug, Clone)]
pub struct FnSignatureGenerator(ast::Fn);

impl GenerateSignature for ast::Fn {
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        use tracing::{debug, trace};

        trace!("Generating signature for ast::Fn with opts: {:?}", opts);

        // 1) Possibly gather doc lines
        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        // 2) Gather visibility, async, etc.
        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let async_str = if let Some(token) = self.async_token() {
            format!("{} ", token.text())
        } else {
            "".to_string()
        };

        let fn_keyword = "fn";
        let name_str = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<anon>".to_string());

        // 3) Generic params
        let generic_params = self
            .generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
            .unwrap_or_default();

        // 4) Parameter list
        let mut param_texts = Vec::new();
        if let Some(plist) = self.param_list() {
            debug!(?plist, "Found param_list for fn");
            if let Some(sp) = plist.self_param() {
                let has_amp = sp.amp_token().is_some();  
                let has_mut = sp.mut_token().is_some();
                let lifetime_str = sp
                    .lifetime()
                    .map(|lt| lt.syntax().text().to_string())
                    .unwrap_or_default();

                let mut pieces = String::new();
                if has_amp {
                    pieces.push('&');
                    if !lifetime_str.is_empty() {
                        pieces.push_str(&lifetime_str);
                        pieces.push(' ');
                    }
                }
                if has_mut {
                    pieces.push_str("mut ");
                }
                pieces.push_str("self");
                param_texts.push(pieces.trim_end().to_string());
            }

            for param in plist.params() {
                if let Some(normal) = ast::Param::cast(param.syntax().clone()) {
                    let pat_str = normal
                        .pat()
                        .map(|p| p.syntax().text().to_string())
                        .unwrap_or_default();
                    let ty_str = normal
                        .ty()
                        .map(|t| t.syntax().text().to_string())
                        .unwrap_or_default();
                    if !pat_str.is_empty() && !ty_str.is_empty() {
                        param_texts.push(format!("{}: {}", pat_str, ty_str));
                    } else if !ty_str.is_empty() {
                        param_texts.push(ty_str);
                    } else if !pat_str.is_empty() {
                        param_texts.push(pat_str);
                    } else {
                        param_texts.push("<unknown_param>".to_string());
                    }
                } else {
                    param_texts.push("<unrecognized_param>".to_string());
                }
            }
        }
        let params_str = param_texts.join(", ");

        // 5) Return type
        let ret_str = if let Some(ret_type) = self.ret_type() {
            if let Some(ty_node) = ret_type.ty() {
                format!(" -> {}", ty_node.syntax().text())
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // 6) Where clause
        let where_str = if let Some(wc) = self.where_clause() {
            format!(" {}", wc.syntax().text())
        } else {
            "".to_string()
        };

        // 7) Build signature line ***WITHOUT*** appending braces.
        //    (We let CrateInterfaceItem<T> decide how to handle the body.)
        let raw_sig = format!(
            "{vis_str}{async_str}{fn_keyword} {name_str}{generic_params}({params_str}){ret_str}{where_str}"
        );

        let combined = match opts.add_semicolon() {
            true  => format!("{doc_text}{raw_sig};"),
            false => format!("{doc_text}{raw_sig}"),
        };

        post_process_spacing(&combined)
    }
}
