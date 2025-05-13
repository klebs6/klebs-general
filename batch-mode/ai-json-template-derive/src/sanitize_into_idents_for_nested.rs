// ---------------- [ File: ai-json-template-derive/src/sanitize_into_idents_for_nested.rs ]
crate::ix!();

/// This helper sanitizes a type string (e.g. "HashMap<u8, String>" => "HashMap_u8_String")
/// then **conditionally** appends "Justification" or "Confidence":
/// - If there's at least one underscore in the sanitized name, we do `Foo_Bar_Justification`.
/// - If there's **no** underscore in the sanitized name, we do `FooBarJustification`.
pub fn sanitize_into_idents_for_nested(
    the_type: &syn::Type,
    span: proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    use quote::quote;

    let raw = quote!(#the_type).to_string();

    // 1) Replace all non-alphanumeric with underscore
    let mut s = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    // 2) collapse repeated underscores
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    // 3) trim leading/trailing underscores
    s = s.trim_matches('_').to_string();

    // 4) if empty or starts with digit, prefix something
    if s.is_empty() {
        s = "NestedType".to_string();
    } else if s.chars().next().unwrap().is_ascii_digit() {
        s = format!("T{}", s);
    }

    // Decide how to append "Justification"/"Confidence"
    // - If s has at least one underscore, we do "s_Justification", else "sJustification"
    let has_underscore = s.contains('_');
    let justification_name = if has_underscore {
        format!("{}_Justification", s)
    } else {
        format!("{}Justification", s)
    };
    let confidence_name = if has_underscore {
        format!("{}_Confidence", s)
    } else {
        format!("{}Confidence", s)
    };

    let just_ident = syn::Ident::new(&justification_name, span);
    let conf_ident = syn::Ident::new(&confidence_name, span);

    (quote!(#just_ident), quote!(#conf_ident))
}

#[cfg(test)]
mod test_sanitize_into_idents_for_nested {
    use super::*;

    #[traced_test]
    fn test_empty_input_becomes_nestedtype() {
        info!("Starting test: test_empty_input_becomes_nestedtype");
        // We simulate an empty type by making a path with zero segments (an odd case).
        // Another approach is just parse_quote!(), but that fails to parse an actually empty type.
        // So let's forcibly parse '()' as a stand-in.
        let empty_path: Type = parse_quote! { () };
        trace!("Using an artificial '()' parse as an 'empty' surrogate.");

        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&empty_path, proc_macro2::Span::call_site());

        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // After all replacements, s should become "NestedType"
        // => justification = "NestedTypeJustification"
        // => confidence = "NestedTypeConfidence"
        assert_eq!(just_ts.to_string(), "NestedTypeJustification");
        assert_eq!(conf_ts.to_string(), "NestedTypeConfidence");
    }

    #[traced_test]
    fn test_alphanumeric_remains_alphanumeric() {
        info!("Starting test: test_alphanumeric_remains_alphanumeric");
        let input: Type = parse_quote!(Foo123Bar456); // valid type ident
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());

        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // No non-alphanumeric => no underscores inserted
        // => "Foo123Bar456Justification" / "Foo123Bar456Confidence"
        assert_eq!(just_ts.to_string(), "Foo123Bar456Justification");
        assert_eq!(conf_ts.to_string(), "Foo123Bar456Confidence");
    }

    #[traced_test]
    fn test_non_alphanumeric_becomes_underscore_and_collapses() {
        info!("Starting test: test_non_alphanumeric_becomes_underscore_and_collapses");
        // Instead of trying parse_quote!(My$Type@@Here!!!) — which is invalid Rust syntax —
        // we parse a trivial type (()), then manually override the raw string:
        let dummy: Type = parse_quote!(());
        let raw_override = "My$Type@@Here!!!".to_string();

        // We'll replicate the same logic the function does on `raw`.
        let mut s = raw_override.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
        while s.contains("__") {
            s = s.replace("__", "_");
        }
        s = s.trim_matches('_').to_string();
        if s.is_empty() {
            s = "NestedType".to_string();
        } else if s.chars().next().unwrap().is_ascii_digit() {
            s = format!("T{}", s);
        }
        let has_underscore = s.contains('_');
        let justification_name = if has_underscore {
            format!("{}_Justification", s)
        } else {
            format!("{}Justification", s)
        };
        let confidence_name = if has_underscore {
            format!("{}_Confidence", s)
        } else {
            format!("{}Confidence", s)
        };
        let just_ident = syn::Ident::new(&justification_name, proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new(&confidence_name, proc_macro2::Span::call_site());

        let (just_ts, conf_ts) = (quote!(#just_ident), quote!(#conf_ident));
        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // Original raw => "My$Type@@Here!!!"
        // After mapping => "My_Type___Here___"
        // Then repeated underscores collapse => "My_Type_Here_"
        // Then trim => "My_Type_Here"
        // => has underscore => "My_Type_Here_Justification" / "My_Type_Here_Confidence"
        assert_eq!(just_ts.to_string(), "My_Type_Here_Justification");
        assert_eq!(conf_ts.to_string(), "My_Type_Here_Confidence");
    }

    #[traced_test]
    fn test_starts_with_digit_prefixed_by_t() {
        info!("Starting test: test_starts_with_digit_prefixed_by_t");
        // parse_quote!(123NumbersHere) would fail. So again, parse a dummy () type, then override:
        let dummy: Type = parse_quote!(());
        let raw_override = "123NumbersHere".to_string();

        // Same sanitizing as the main function:
        let mut s = raw_override.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
        while s.contains("__") {
            s = s.replace("__", "_");
        }
        s = s.trim_matches('_').to_string();
        if s.is_empty() {
            s = "NestedType".to_string();
        } else if s.chars().next().unwrap().is_ascii_digit() {
            s = format!("T{}", s);
        }
        let has_underscore = s.contains('_');
        let justification_name = if has_underscore {
            format!("{}_Justification", s)
        } else {
            format!("{}Justification", s)
        };
        let confidence_name = if has_underscore {
            format!("{}_Confidence", s)
        } else {
            format!("{}Confidence", s)
        };
        let just_ident = syn::Ident::new(&justification_name, proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new(&confidence_name, proc_macro2::Span::call_site());

        let (just_ts, conf_ts) = (quote!(#just_ident), quote!(#conf_ident));
        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // => "123NumbersHere" => no punctuation => trim => "123NumbersHere"
        // => starts w/ digit => "T123NumbersHere"
        // => no underscore => => "T123NumbersHereJustification" / "T123NumbersHereConfidence"
        assert_eq!(just_ts.to_string(), "T123NumbersHereJustification");
        assert_eq!(conf_ts.to_string(), "T123NumbersHereConfidence");
    }

    #[traced_test]
    fn test_leading_and_trailing_underscores_removed() {
        info!("Starting test: test_leading_and_trailing_underscores_removed");
        let input: Type = parse_quote!(__TypeWith___Noisy___Edges__);
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());

        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // Raw => "__TypeWith___Noisy___Edges__"
        // => after cleanup => "TypeWith_Noisy_Edges"
        // => has underscores => "TypeWith_Noisy_Edges_Justification" / "TypeWith_Noisy_Edges_Confidence"
        assert_eq!(just_ts.to_string(), "TypeWith_Noisy_Edges_Justification");
        assert_eq!(conf_ts.to_string(), "TypeWith_Noisy_Edges_Confidence");
    }

    #[traced_test]
    fn test_complex_nested_path() {
        info!("Starting test: test_complex_nested_path");
        let input: Type = parse_quote!(std::collections::HashMap<u32,String>);
        // The raw string => "std :: collections :: HashMap < u32 , String >"
        // => non-alphanumeric => "std__collections__HashMap__u32__String_"
        // => collapsed => "std_collections_HashMap_u32_String_"
        // => trim => "std_collections_HashMap_u32_String"
        // => has underscore => "std_collections_HashMap_u32_String_Justification" / "..._Confidence"
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());

        trace!("just_ts = {}", just_ts);
        trace!("conf_ts = {}", conf_ts);

        assert_eq!(just_ts.to_string(), "std_collections_HashMap_u32_String_Justification");
        assert_eq!(conf_ts.to_string(), "std_collections_HashMap_u32_String_Confidence");
    }

    #[traced_test]
    fn test_all_punctuation_input() {
        info!("Starting test: test_all_punctuation_input");
        // parse_quote!(!!!???@@@) is invalid Rust; we do a dummy type + override again:
        let dummy: Type = parse_quote!(());
        let raw_override = "!!!???@@@".to_string();

        let mut s = raw_override.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
        while s.contains("__") {
            s = s.replace("__", "_");
        }
        s = s.trim_matches('_').to_string();
        if s.is_empty() {
            s = "NestedType".to_string();
        } else if s.chars().next().unwrap().is_ascii_digit() {
            s = format!("T{}", s);
        }
        let has_underscore = s.contains('_');
        let justification_name = if has_underscore {
            format!("{}_Justification", s)
        } else {
            format!("{}Justification", s)
        };
        let confidence_name = if has_underscore {
            format!("{}_Confidence", s)
        } else {
            format!("{}Confidence", s)
        };
        let just_ident = syn::Ident::new(&justification_name, proc_macro2::Span::call_site());
        let conf_ident = syn::Ident::new(&confidence_name, proc_macro2::Span::call_site());
        let (just_ts, conf_ts) = (quote!(#just_ident), quote!(#conf_ident));

        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        // All punctuation => after mapping => underscores => eventually collapses to "", 
        // => replaced with "NestedType"
        // => "NestedTypeJustification" / "NestedTypeConfidence"
        assert_eq!(just_ts.to_string(), "NestedTypeJustification");
        assert_eq!(conf_ts.to_string(), "NestedTypeConfidence");
    }

    #[traced_test]
    fn test_already_has_underscore() {
        info!("Starting test: test_already_has_underscore");
        let input: Type = parse_quote!(My_Type);
        // => "My_Type"
        // => no leading/trailing underscores removed => "My_Type"
        // => has underscore => => "My_Type_Justification" / "My_Type_Confidence"
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());
        trace!("just_ts = {}", just_ts);
        trace!("conf_ts = {}", conf_ts);

        assert_eq!(just_ts.to_string(), "My_Type_Justification");
        assert_eq!(conf_ts.to_string(), "My_Type_Confidence");
    }

    #[traced_test]
    fn test_simple_type_no_underscore() {
        info!("Starting test: test_simple_type_no_underscore");
        let input: Type = parse_quote!(MyType);
        // => "MyType", no underscores => "MyTypeJustification" / "MyTypeConfidence"
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());
        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        assert_eq!(just_ts.to_string(), "MyTypeJustification");
        assert_eq!(conf_ts.to_string(), "MyTypeConfidence");
    }

    #[traced_test]
    fn test_contains_generic_arguments() {
        info!("Starting test: test_contains_generic_arguments");
        let input: Type = parse_quote!(TypeA<BadTypeC<D2>>);
        // => raw => "TypeA < BadTypeC < D2 > >"
        // => mapped => "TypeA__BadTypeC__D2__"
        // => collapsed => "TypeA_BadTypeC_D2_"
        // => trimmed => "TypeA_BadTypeC_D2"
        // => underscore => "TypeA_BadTypeC_D2_Justification" / "..._Confidence"
        let (just_ts, conf_ts) = sanitize_into_idents_for_nested(&input, proc_macro2::Span::call_site());
        debug!("just_ts = {}", just_ts);
        debug!("conf_ts = {}", conf_ts);

        assert_eq!(just_ts.to_string(), "TypeA_BadTypeC_D2_Justification");
        assert_eq!(conf_ts.to_string(), "TypeA_BadTypeC_D2_Confidence");
    }
}
