crate::ix!();

pub fn gather_named_struct_just_conf_idents(
    ty_ident: &syn::Ident,
    span: proc_macro2::Span,
) -> (syn::Ident, syn::Ident, syn::Ident) {
    trace!(
        "Constructing justification/conf/justified idents for '{}'",
        ty_ident
    );
    let justification_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident = syn::Ident::new(&format!("{}Confidence", ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);
    debug!(
        " -> justification_ident='{}', confidence_ident='{}', justified_ident='{}'",
        justification_ident, confidence_ident, justified_ident
    );
    (justification_ident, confidence_ident, justified_ident)
}
