// ---------------- [ File: src/generate_impl_signature.rs ]
crate::ix!();

// Return something like "impl SomeTrait for T" WITHOUT braces
pub fn generate_impl_signature(impl_ast: &ast::Impl, docs: Option<&String>) -> String {
    let doc_part = docs
        .filter(|d| !d.trim().is_empty())
        .map(|d| format!("{d}\n"))
        .unwrap_or_default();

    let generic_params = impl_ast
        .generic_param_list()
        .map(|gp| gp.syntax().text().to_string())
        .unwrap_or_default();
    let where_clause = impl_ast
        .where_clause()
        .map(|wc| wc.syntax().text().to_string())
        .unwrap_or_default();
    let trait_part = impl_ast
        .trait_()
        .map_or("".to_string(), |tr| tr.syntax().text().to_string());
    let self_ty = impl_ast
        .self_ty()
        .map_or("???".to_string(), |ty| ty.syntax().text().to_string());

    let signature_line = if trait_part.is_empty() {
        format!("impl{generic_params} {self_ty} {where_clause}")
    } else {
        format!("impl{generic_params} {trait_part} for {self_ty} {where_clause}")
    };

    format!("{doc_part}{signature_line}")
}
