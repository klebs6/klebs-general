// ---------------- [ File: ai-json-template-derive/src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    tracing::trace!("classify_field_type => doc_str={:?}, type=? => Checking type for AiJsonTemplate", doc_str);

    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // 1) If it's an Option<T>, treat T as not required
    if let Some(inner_ty) = extract_option_inner(ty) {
        tracing::trace!("Field is Option<...> => required=false");
        return emit_schema_for_type(inner_ty, doc_lit, false);
    }

    // Otherwise required=true
    tracing::trace!("Field is not an Option => required=true");
    emit_schema_for_type(ty, doc_lit, true)
}
