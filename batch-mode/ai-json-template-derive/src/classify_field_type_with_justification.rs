// ---------------- [ File: ai-json-template-derive/src/classify_field_type_with_justification.rs ]
crate::ix!();

pub fn classify_field_type_with_justification(
    ty: &syn::Type,
    doc_str: &str,
    required: bool,
) -> Option<proc_macro2::TokenStream> {
    trace!("classify_field_type_with_justification => type: {:?}, required: {}, doc_str: {:?}",
           ty, required, doc_str);

    // Because doc_str can have extra whitespace, we'll trim it for clarity:
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());
    let required_bool = quote::quote!(#required);

    // 1) If it's Option<T>, handle T as not required
    if let Some(inner) = extract_option_inner(ty) {
        trace!("Detected Option<T>");
        return build_option_schema(inner, doc_str);
    }

    // 2) If it's Vec<T>, handle array_of
    if let Some(elem_ty) = extract_vec_inner(ty) {
        trace!("Detected Vec<T>");
        return build_vec_schema(elem_ty, required_bool.clone(), doc_lit.clone());
    }

    // 3) If it's HashMap<K, V>, decide how to represent K and V
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        trace!("Detected HashMap<K, V>");
        return build_hashmap_schema(k_ty, v_ty, required_bool.clone(), doc_lit.clone());
    }

    // 4) Builtin bool => "boolean"
    if is_bool(ty) {
        trace!("Detected bool => 'boolean'");
        return build_bool_schema(required_bool, doc_lit);
    }

    // 5) Builtin String => "string"
    if is_string_type(ty) {
        trace!("Detected String => 'string'");
        return build_string_schema(required_bool, doc_lit);
    }

    // 6) Builtin numeric => "number"
    if is_numeric(ty) {
        trace!("Detected numeric => 'number'");
        return build_numeric_schema(required_bool, doc_lit);
    }

    // 7) Otherwise => nested struct or enum => call AiJsonTemplateWithJustification
    trace!("Treating as nested struct/enum => calling AiJsonTemplateWithJustification");
    build_nested_schema(ty, required_bool, doc_lit)
}
