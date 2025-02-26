// ---------------- [ File: src/generate_function_signature.rs ]
crate::ix!();

/// Minimal post-processing for spacing around `->` and `where`.
/// For example, it ensures `) ->` has a space, and `> where` has a space.
pub fn post_process_spacing(signature: &str) -> String {
    signature
        .replace(")->", ") ->")
        .replace(">where", "> where")
}

impl GenerateSignature for ast::Fn {
    fn generate_signature(&self) -> String {
        // For clarity, we omit doc lines, etc. The key is the parameter logic below.

        // 1) Gather visibility and async tokens (unchanged from your code)
        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();
        let async_str = if let Some(token) = self.async_token() {
            format!("{} ", token.text())
        } else {
            "".to_string()
        };

        // 2) The function name
        let fn_keyword = "fn";
        let name_str = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<anon>".to_string());

        // 3) Generic params, e.g. "<T>"
        let generic_params = self
            .generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
            .unwrap_or_default();

        // 4) Build a vector of parameter texts
        let mut param_texts = Vec::new();

        // (A) If we have a param_list node, check for self_param, then normal params:
        if let Some(plist) = self.param_list() {
            // For debugging if you like:
            debug!("DEBUG: param_list = {}", plist.syntax().text());

            // ---- STEP A1: Check for a "self_param"
            if let Some(sp) = plist.self_param() {
                // This is where `&self` or `&mut self` or just `self` is stored
                debug!("DEBUG: found self param syntax = '{}'", sp.syntax().text());

                let has_amp = sp.amp_token().is_some();  
                let has_mut = sp.mut_token().is_some();
                let lifetime_str = sp
                    .lifetime()
                    .map(|lt| lt.syntax().text().to_string())
                    .unwrap_or_default();

                // build e.g. "&'a mut self"
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

            // ---- STEP A2: For the rest of the parameters
            // (These do NOT include self_param())
            for param in plist.params() {
                debug!("DEBUG: normal param syntax = '{}'", param.syntax().text());

                if let Some(normal) = ast::Param::cast(param.syntax().clone()) {
                    // parse pattern + type
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
                    // In practice, we rarely hit this, but just in case:
                    param_texts.push("<unrecognized_param>".to_string());
                }
            }
        }

        // 5) Join the param texts
        let params_str = param_texts.join(", ");

        // 6) Return type
        let ret_str = if let Some(ret_type) = self.ret_type() {
            if let Some(ty_node) = ret_type.ty() {
                format!(" -> {}", ty_node.syntax().text())
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // 7) Where clause
        let where_str = if let Some(wc) = self.where_clause() {
            format!(" {}", wc.syntax().text())
        } else {
            "".to_string()
        };

        // Build final
        let raw_sig = format!(
            "{vis_str}{async_str}{fn_keyword} {name_str}{generic_params}({params_str}){ret_str}{where_str}"
        );
        post_process_spacing(&raw_sig)
    }
}
