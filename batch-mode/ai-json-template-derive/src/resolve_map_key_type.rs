// ---------------- [ File: ai-json-template-derive/src/resolve_map_key_type.rs ]
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

#[cfg(test)]
mod verify_resolve_map_key_type {
    use super::*;

    #[traced_test]
    fn handle_bool_key_should_error() {
        trace!("Testing `bool` key => should produce an error TokenStream");
        let bool_type: Type = parse_quote!(bool);

        let result = resolve_map_key_type(&bool_type);
        match result {
            Ok(_) => {
                error!("Expected error for `bool` key, but got Ok(...)");
                panic!("resolve_map_key_type did NOT fail for bool key!");
            }
            Err(err_ts) => {
                info!("Successfully produced error TokenStream for bool key => {:?}", err_ts);
                assert!(true, "Correctly errored on bool key.");
            }
        }
    }

    #[traced_test]
    fn handle_numeric_keys() {
        trace!("Testing numeric keys => should produce Ok(\"number\")");
        let numeric_samples: &[Type] = &[
            parse_quote!(i8),
            parse_quote!(i16),
            parse_quote!(i32),
            parse_quote!(i64),
            parse_quote!(u8),
            parse_quote!(u16),
            parse_quote!(u32),
            parse_quote!(u64),
            parse_quote!(f32),
            parse_quote!(f64),
        ];

        for sample in numeric_samples {
            debug!("Checking numeric type: {:?}", sample);
            let result = resolve_map_key_type(sample);
            match result {
                Ok(ref key_str) => {
                    info!("Key type => {}", key_str);
                    assert_eq!(
                        key_str, "number",
                        "Expected 'number' for numeric key, got: {}",
                        key_str
                    );
                }
                Err(err_ts) => {
                    error!("Unexpected error TokenStream for numeric key: {:?}", err_ts);
                    panic!("Did not expect an error for numeric key.");
                }
            }
        }
    }

    #[traced_test]
    fn handle_string_key() {
        trace!("Testing `String` key => should produce Ok(\"string\")");
        let string_type: Type = parse_quote!(String);

        let result = resolve_map_key_type(&string_type);
        match result {
            Ok(key_str) => {
                info!("Key type => {}", key_str);
                assert_eq!(key_str, "string", "Expected 'string' for String key.");
            }
            Err(err_ts) => {
                error!("Got unexpected error for String key => {:?}", err_ts);
                panic!("Expected success for String key, but got error.");
            }
        }
    }

    #[traced_test]
    fn handle_fallback_custom_key() {
        trace!("Testing fallback for a custom user-defined key => should produce Ok(\"nested_struct_or_enum\")");
        let custom_type: Type = parse_quote!(MyCustomKeyType);

        let result = resolve_map_key_type(&custom_type);
        match result {
            Ok(key_str) => {
                info!("Key type => {}", key_str);
                assert_eq!(
                    key_str, "nested_struct_or_enum",
                    "Expected 'nested_struct_or_enum' for custom key, got: {}",
                    key_str
                );
            }
            Err(err_ts) => {
                error!("Unexpected error for custom key => {:?}", err_ts);
                panic!("Expected success for custom key, but got error.");
            }
        }
    }
}
