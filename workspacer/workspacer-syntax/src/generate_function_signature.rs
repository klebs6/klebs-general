// ---------------- [ File: workspacer-syntax/src/generate_function_signature.rs ]
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

        // 4) Gather parameter entries (including self)
        let mut param_entries = Vec::new();
        if let Some(plist) = self.param_list() {
            debug!(?plist, "Found param_list for fn");

            // Possibly handle "self"
            if let Some(sp) = plist.self_param() {
                let has_amp = sp.amp_token().is_some();
                let has_mut = sp.mut_token().is_some();
                let lifetime_str = sp
                    .lifetime()
                    .map(|lt| lt.syntax().text().to_string())
                    .unwrap_or_default();

                let mut name_part = String::new();
                if has_amp {
                    name_part.push('&');
                    if !lifetime_str.is_empty() {
                        name_part.push_str(&lifetime_str);
                        name_part.push(' ');
                    }
                }
                if has_mut {
                    name_part.push_str("mut ");
                }
                name_part.push_str("self");

                // For "self", treat it as name="self", type="".
                param_entries.push((name_part.trim_end().to_string(), "".to_string()));
            }

            // The rest of the param_list
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
                        param_entries.push((pat_str, ty_str));
                    } else if !ty_str.is_empty() {
                        param_entries.push((ty_str, "".to_string()));
                    } else if !pat_str.is_empty() {
                        param_entries.push((pat_str, "".to_string()));
                    } else {
                        param_entries.push(("<unknown_param>".to_string(), "".to_string()));
                    }
                } else {
                    param_entries.push(("<unrecognized_param>".to_string(), "".to_string()));
                }
            }
        }

        let param_count = param_entries.len();

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
        let where_str = full_clean_where_clause(&self.where_clause());

        // 7) Build final lines
        //    We'll produce something like:
        //
        //    <docs?>
        //    pub async fn name<T>(...) -> ...
        //    or multiline version, but with the opening "(" on the same line as "name<T>"

        // Prefix = "pub async fn name<T>"
        let prefix_line = format!("{vis_str}{async_str}{fn_keyword} {name_str}{generic_params}");

        // If we have <= 3 parameters, do a single-line param string:
        let multiline: bool = param_count > 3;

        let mut param_str = String::new();
        if param_count == 0 {
            // no params => "()"
            param_str.push_str("()");
        } else if !multiline {
            // single-line approach
            let joined: Vec<String> = param_entries
                .iter()
                .map(|(n, t)| {
                    if t.is_empty() {
                        n.to_string()
                    } else {
                        format!("{}: {}", n, t)
                    }
                })
                .collect();
            param_str.push('(');
            param_str.push_str(&joined.join(", "));
            param_str.push(')');
        } else {
            // multiline approach => align
            // 1) find longest name part
            let max_name_len = param_entries
                .iter()
                .map(|(n, _)| n.len())
                .max()
                .unwrap_or(0);

            // open paren on the same line
            param_str.push('(');

            // next lines for each param
            for (i, (name_part, ty_part)) in param_entries.iter().enumerate() {
                param_str.push('\n');
                // indent 4 spaces
                param_str.push_str("    ");
                let spacing_needed = max_name_len.saturating_sub(name_part.len());
                if ty_part.is_empty() {
                    // e.g. "self"
                    param_str.push_str(name_part);
                    // trailing comma
                    if i + 1 < param_count {
                        param_str.push(',');
                    }
                } else {
                    // e.g. "name: Type"
                    param_str.push_str(name_part);
                    param_str.push_str(": ");
                    param_str.push_str(&" ".repeat(spacing_needed));
                    param_str.push_str(ty_part);
                    // trailing comma
                    if i + 1 < param_count {
                        param_str.push(',');
                    }
                }
            }
            param_str.push('\n');
            param_str.push(')');
        }

        // If we have a where clause, attach it after ret_str
        // So e.g. " -> i32 where T: Debug"
        let suffix = if !where_str.is_empty() {
            format!("{ret_str} {where_str}")
        } else {
            ret_str
        };

        // If multiline, we typically place suffix on the same line as ")"
        // but we ended up with no trailing place. Let's see:
        // We'll build the final line: param_str + suffix
        let final_func_line = if multiline {
            // Insert suffix right after the closing parenthesis
            // Possibly check if param_str ends with ')' or not
            if suffix.trim().is_empty() {
                param_str
            } else {
                format!("{param_str}{suffix}")
            }
        } else {
            // single-line => e.g. "(self, x: i32) -> i32 where T: Debug"
            format!("{}{}", param_str, suffix)
        };

        // Combine everything. If user wants a semicolon appended, do so:
        let raw_sig = format!("{prefix_line}{final_func_line}");

        let combined = match opts.add_semicolon() {
            true => format!("{doc_text}{raw_sig};"),
            false => format!("{doc_text}{raw_sig}"),
        };

        post_process_spacing(&combined)
    }
}
