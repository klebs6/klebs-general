crate::ix!();

pub fn resolve_map_key_type(k_ty: &syn::Type) -> Result<String, proc_macro2::TokenStream> {
    trace!("resolve_map_key_type invoked");
    if is_bool(k_ty) {
        let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplate");
        let err = syn::Error::new(k_ty.span(), &err_msg);
        return Err(err.to_compile_error());
    } else if is_numeric(k_ty) {
        Ok("number".to_string())
    } else if is_string_type(k_ty) {
        Ok("string".to_string())
    } else {
        // fallback => treat as nested struct/enum
        Ok("nested_struct_or_enum".to_string())
    }
}
