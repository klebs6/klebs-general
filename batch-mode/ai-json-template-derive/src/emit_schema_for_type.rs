// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_type.rs ]
crate::ix!();

pub fn emit_schema_for_type(
    ty: &syn::Type,
    doc_lit: proc_macro2::Literal,
    required: bool
) -> Option<proc_macro2::TokenStream> {
    trace!("Starting emit_schema_for_type");

    let required_bool = if required {
        quote!(true)
    } else {
        quote!(false)
    };
    let generation_instructions = format!("{}", doc_lit);
    let type_str = quote!(#ty).to_string();

    trace!(
        "emit_schema_for_type => required={} type={}",
        required,
        type_str
    );

    // (A) Handle bool, string, numeric
    if is_bool(ty) {
        return Some(emit_schema_for_bool(&generation_instructions, &required_bool));
    }
    if is_string_type(ty) {
        return Some(emit_schema_for_string(&generation_instructions, &required_bool));
    }
    if is_numeric(ty) {
        return Some(emit_schema_for_number(&generation_instructions, &required_bool));
    }

    // (B) Handle Vec<T>
    if let Some(elem_ty) = extract_vec_inner(ty) {
        debug!("Detected Vec<T> => specialized array-of expansions");
        return Some(emit_schema_for_vec(
            elem_ty,
            &generation_instructions,
            &required_bool,
        ));
    }

    // (C) Handle HashMap<K, V>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        debug!("Detected HashMap => specialized expansions for map");
        return Some(emit_schema_for_hashmap(
            k_ty,
            v_ty,
            &generation_instructions,
            &required_bool,
        ));
    }

    // (D) Fallback => treat as a nested struct or enum
    trace!(
        "Falling back to nested struct/enum expansion for type={}",
        type_str
    );
    Some(emit_schema_for_fallback_nested(
        ty,
        &generation_instructions,
        &required_bool,
    ))
}
