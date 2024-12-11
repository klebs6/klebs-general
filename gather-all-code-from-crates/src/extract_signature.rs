crate::ix!();

pub fn extract_signature(fn_def: &ast::Fn, remove_doc_comments: bool) -> String {
    let vis_str = fn_def.visibility()
        .map(|v| v.syntax().text().to_string() + " ")
        .unwrap_or_default();

    let name_str = fn_def.name()
        .map(|n| n.text().to_string())
        .unwrap_or_default();

    let params_str = fn_def.param_list()
        .map(|p| p.syntax().text().to_string())
        .unwrap_or_else(|| "()".to_string());

    let ret_str = fn_def.ret_type()
        .map(|r| format!(" {}", r.syntax().text()))
        .unwrap_or_default();

    let mut signature = format!("{}fn {}{}{}", vis_str, name_str, params_str, ret_str);

    if remove_doc_comments {
        // Remove doc comments from the signature line if any appear (unlikely if we build purely from AST)
        signature = filter_doc_comments(&signature, true);
    }

    signature.trim_end().to_string()
}
