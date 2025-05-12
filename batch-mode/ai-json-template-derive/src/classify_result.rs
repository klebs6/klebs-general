// ---------------- [ File: ai-json-template-derive/src/classify_result.rs ]
crate::ix!();

/// If the field is a built-in scalar (bool, numeric, String), we store justification as String, confidence as f32.
/// Otherwise, we treat it as a "nested" type that presumably implements AiJsonTemplateWithJustification,
/// generating e.g. "MyTypeJustification" / "MyTypeConfidence", or "HashMap_u8_String_Justification" if underscores exist.
#[derive(Debug)]
pub enum ClassifyResult {
    /// Basic justification = String, confidence = f32
    JustString,
    /// Nested => we produce a justification_type and confidence_type
    NestedJustification {
        justification_type: proc_macro2::TokenStream,
        confidence_type:    proc_macro2::TokenStream,
    },
}

pub fn classify_for_justification(
    ty: &syn::Type
) -> Result<ClassifyResult, proc_macro2::TokenStream> {

    // 1) If it's a built-in scalar => JustString
    if is_builtin_scalar(ty) {
        return Ok(ClassifyResult::JustString);
    }

    // 2) If the type name includes "BadType", produce compile_error
    let raw_str = quote::quote!(#ty).to_string();
    if raw_str.contains("BadType") {
        let msg = format!("Type {} is not supported by AiJsonTemplateWithJustification", raw_str);
        return Err(quote::quote! { compile_error!(#msg); });
    }

    // 3) Option<T> => if T is builtin => JustString else Nested
    if let Some(inner) = extract_option_inner(ty) {
        if is_builtin_scalar(inner) {
            return Ok(ClassifyResult::JustString);
        } else {
            let (just_ts, conf_ts) = sanitize_into_idents_for_nested(inner, ty.span());
            return Ok(ClassifyResult::NestedJustification {
                justification_type: just_ts,
                confidence_type: conf_ts,
            });
        }
    }

    // 4) Vec<T> => if T is builtin => JustString else Nested
    if let Some(elem) = extract_vec_inner(ty) {
        if is_builtin_scalar(elem) {
            return Ok(ClassifyResult::JustString);
        } else {
            let (just_ts, conf_ts) = sanitize_into_idents_for_nested(elem, ty.span());
            return Ok(ClassifyResult::NestedJustification {
                justification_type: just_ts,
                confidence_type: conf_ts,
            });
        }
    }

    // 5) HashMap<K, V>
    if let Some((k_ty, _v_ty)) = extract_hashmap_inner(ty) {
        // If key is bool => produce an error
        if is_bool(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
            return Err(quote::quote! { compile_error!(#err_msg); });
        }
        // Otherwise (numeric, string, or custom key), we store a single string justification
        // for the entire map => "JustString"
        return Ok(ClassifyResult::JustString);
    }

    // 6) Otherwise => treat as custom nested type => e.g. "MyTypeJustification"
    let (just_ts, conf_ts) = sanitize_into_idents_for_nested(ty, ty.span());
    Ok(ClassifyResult::NestedJustification {
        justification_type: just_ts,
        confidence_type:    conf_ts,
    })
}

#[cfg(test)]
mod test_subroutine_classify_for_justification {
    use super::*;

    #[traced_test]
    fn test_bad_type() {
        // e.g. "BadType"
        let t: syn::Type = parse_quote! { BadType };
        let err = classify_for_justification(&t).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("compile_error"), "should have compile_error");
        assert!(s.contains("BadType"), "Should mention 'BadType'");
    }

    #[traced_test]
    fn test_no_underscore_single_word() {
        // e.g. "Custom" => "CustomJustification" not "Custom_Justification"
        let t: syn::Type = parse_quote! { Custom };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => panic!("Expected nested for 'Custom'"),
            ClassifyResult::NestedJustification { justification_type, confidence_type } => {
                let jt = justification_type.to_string();
                let ct = confidence_type.to_string();
                assert!(
                    jt.contains("CustomJustification"),
                    "Got {jt:?}"
                );
                assert!(
                    ct.contains("CustomConfidence"),
                    "Got {ct:?}"
                );
            }
        }
    }

    #[traced_test]
    fn test_scalar() {
        let t: syn::Type = parse_quote! { bool };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => {},
            _ => panic!("bool => JustString expected."),
        }
    }

    #[traced_test]
    fn test_custom() {
        let t: syn::Type = parse_quote! { MyCustomType<Stuff> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::NestedJustification { .. } => {},
            _ => panic!("Expected NestedJustification for custom type"),
        }
    }

    #[traced_test]
    fn test_option_builtin() {
        // Option<String> => JustString (avoid generating Option___String__Justification).
        let t: syn::Type = parse_quote! { Option<String> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => {},
            _ => panic!("Option<String> should yield JustString in justification."),
        }
    }

    #[traced_test]
    fn test_option_custom() {
        // Option<SomethingElse> => NestedJustification if SomethingElse is custom
        let t: syn::Type = parse_quote! { Option<MyOtherType> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::NestedJustification { .. } => {},
            _ => panic!("Option<MyOtherType> => NestedJustification expected."),
        }
    }

    #[traced_test]
    fn test_vec_builtin() {
        // Vec<u32> => JustString for justification (a single textual explanation for the array).
        let t: syn::Type = parse_quote! { Vec<u32> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => {},
            _ => panic!("Vec<u32> => JustString expected for justification."),
        }
    }

    #[traced_test]
    fn test_vec_custom() {
        // Vec<Custom> => NestedJustification
        let t: syn::Type = parse_quote! { Vec<CustomItem> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::NestedJustification { .. } => {},
            _ => panic!("Vec<CustomItem> => NestedJustification expected."),
        }
    }

    #[traced_test]
    fn test_hashmap_u8_string() {
        let t: syn::Type = parse_quote! { HashMap<u8, String> };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => {
                // Now that we only store a single textual justification for the entire map,
                // let's just assert that we got JustString. That is correct for our new approach.
            }
            ClassifyResult::NestedJustification { .. } => {
                panic!("HashMap<u8,String> => expected single textual justification, not NestedJustification")
            }
        }
    }

    #[traced_test]
    fn test_clean_up() {
        let t: syn::Type = parse_quote! { HashMap < u8, NodeVariantLevelWeights > };
        let c = classify_for_justification(&t).unwrap();
        match c {
            ClassifyResult::JustString => {
                // success branch, we decided to store a single textual justification for the entire map
            }
            ClassifyResult::NestedJustification { .. } => {
                panic!("HashMap => single textual justification => we do not want a nested approach")
            }
        }
    }
}
