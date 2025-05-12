// ---------------- [ File: ai-json-template-derive/src/create_flat_justified_idents_for_named.rs ]
crate::ix!();

pub fn create_flat_justified_idents_for_named(
    ty_ident: &syn::Ident,
    span: proc_macro2::Span,
) -> (syn::Ident, syn::Ident, syn::Ident, syn::Ident) {
    trace!("create_flat_justified_idents_for_named: starting for '{}'", ty_ident);

    let flat_ident = syn::Ident::new(&format!("FlatJustified{}", ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);
    let justification_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident = syn::Ident::new(&format!("{}Confidence", ty_ident), span);

    debug!("create_flat_justified_idents_for_named => flat_ident='{}', justified_ident='{}', justification_ident='{}', confidence_ident='{}'",
        flat_ident, justified_ident, justification_ident, confidence_ident
    );
    (flat_ident, justified_ident, justification_ident, confidence_ident)
}
