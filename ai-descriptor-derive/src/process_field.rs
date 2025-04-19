// ---------------- [ File: ai-descriptor-derive/src/process_field.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub(crate) fn process_field(field: &Field) -> Result<TokenStream2, TokenStream2> {

    use tracing::{trace, debug, info, warn, error};

    /// A small helper struct storing field-level ai-attributes:
    /// - `feature_if_none`
    /// - `feature_prefix`
    /// - `feature_postfix`
    /// - `will_use_ai_alt`
    #[derive(Debug, Default)]
    struct FieldAiInfo {
        feature_if_none: Option<String>,
        feature_prefix:  Option<String>,
        feature_postfix: Option<String>,
        will_use_ai_alt: bool,
    }

    /// Parse the relevant `#[ai(...)]` attributes on this field.
    /// We might see:
    ///   #[ai(feature_if_none = "...")]
    ///   #[ai(feature_prefix = "...")]
    ///   #[ai(feature_postfix = "...")]
    ///   #[ai(will_use_ai_alt = true)]
    #[tracing::instrument(level="trace", skip_all)]
    fn parse_field_ai_attributes(attrs: &[syn::Attribute]) -> syn::Result<FieldAiInfo> {
        let mut info = FieldAiInfo::default();

        for attr in attrs {
            if attr.path.is_ident("ai") {
                if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in &meta_list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = nested {
                            let key = &nv.path;
                            if key.is_ident("feature_if_none") {
                                if let syn::Lit::Str(ref lit_str) = nv.lit {
                                    info.feature_if_none = Some(lit_str.value());
                                }
                            } else if key.is_ident("feature_prefix") {
                                if let syn::Lit::Str(ref lit_str) = nv.lit {
                                    info.feature_prefix = Some(lit_str.value());
                                }
                            } else if key.is_ident("feature_postfix") {
                                if let syn::Lit::Str(ref lit_str) = nv.lit {
                                    info.feature_postfix = Some(lit_str.value());
                                }
                            } else if key.is_ident("will_use_ai_alt") {
                                if let syn::Lit::Bool(ref lit_bool) = nv.lit {
                                    info.will_use_ai_alt = lit_bool.value();
                                } else {
                                    let msg = "will_use_ai_alt must be a boolean literal, e.g. will_use_ai_alt = true";
                                    error!("{}", msg);
                                    return Err(syn::Error::new_spanned(&nv.lit, msg));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(info)
    }

    trace!("Entering process_field for field: {:?}", field.ident);

    // 1) Identify the field name and parse the field-level attributes
    let field_name = match &field.ident {
        Some(ident) => ident,
        None => {
            let msg = "Unnamed field in process_field; ItemWithFeatures expects named fields";
            error!("{}", msg);
            return Err(quote_spanned!(field.span()=> compile_error!(#msg); ));
        }
    };

    let field_info = match parse_field_ai_attributes(&field.attrs) {
        Ok(info) => info,
        Err(e) => {
            let msg = format!("Failed to parse #[ai(...)] attributes for field '{}'", field_name);
            error!("{}", msg);
            return Err(e.to_compile_error());
        }
    };

    // 2) Check if the field is Option<T>
    let is_opt = crate::is_option_type(&field.ty);

    // 3) If non-optional and `feature_if_none` is set, produce an error
    if !is_opt && field_info.feature_if_none.is_some() {
        let span = field.span();
        let msg = "The `feature_if_none` attribute is only applicable to Option types";
        error!("Field '{}' is non-optional but used `feature_if_none` => not allowed", field_name);
        return Err(quote_spanned!(span=> compile_error!(#msg); ));
    }

    let prefix = field_info.feature_prefix.unwrap_or_default();
    let postfix = field_info.feature_postfix.unwrap_or_default();

    // If `will_use_ai_alt` is true, we call `.ai_alt()`; otherwise, we call `.text()`.
    // We'll define a tiny helper for that choice.
    #[tracing::instrument(level="trace", skip_all)]
    fn field_method_call(will_use_ai_alt: bool) -> TokenStream2 {
        if will_use_ai_alt {
            quote! { ai_alt() }
        } else {
            quote! { text() }
        }
    }

    let method_call = field_method_call(field_info.will_use_ai_alt);

    // We'll generate code that yields `original_text` from the field
    // or from the default string if `feature_if_none` is present. Then we combine prefix/postfix.
    let produce_combined = |value_expr: proc_macro2::TokenStream| {
        quote! {
            {
                let prefix_space = if #prefix.is_empty() {
                    std::string::String::new()
                } else {
                    format!("{} ", #prefix)
                };
                let postfix_space = if #postfix.is_empty() {
                    std::string::String::new()
                } else {
                    format!(" {}", #postfix)
                };
                let combined = format!("{}{}{}", prefix_space, #value_expr, postfix_space);
                features.push(std::borrow::Cow::Owned(combined));
            }
        }
    };

    if is_opt {
        // If the field is Option<T>
        if let Some(default_text) = field_info.feature_if_none {
            // We produce a match
            //   match &self.field_name {
            //       Some(value) => { prefix + value.(text or ai_alt) + postfix }
            //       None => { prefix + default_text + postfix }
            //   }
            let some_arm = produce_combined(quote!(value.#method_call));
            let none_arm = produce_combined(quote!(std::borrow::Cow::Borrowed(#default_text)));
            Ok(quote! {
                match &self.#field_name {
                    Some(value) => #some_arm,
                    None => #none_arm,
                }
            })
        } else {
            // We produce an if-let
            //   if let Some(value) = &self.field_name {
            //       prefix + value.(text or ai_alt) + postfix
            //   }
            let body = produce_combined(quote!(value.#method_call));
            Ok(quote! {
                if let Some(value) = &self.#field_name {
                    #body
                }
            })
        }
    } else {
        // Non-optional. We directly push self.field_name.(text or ai_alt)
        let body = produce_combined(quote!(self.#field_name.#method_call));
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Field, ItemStruct, Fields};
    use quote::ToTokens;
    use traced_test::traced_test;
    use tracing::{trace, debug, info, warn, error};

    /// Convert code to a "no_whitespace" string for easy substring checks.
    fn no_ws(s: &str) -> String {
        s.replace(' ', "")
         .replace('\n',"")
         .replace('\r',"")
         .replace('\t',"")
    }

    /// Extract the single named field from a struct definition
    fn extract_first_named_field(item_struct: ItemStruct) -> Field {
        match &item_struct.fields {
            Fields::Named(named) => named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        }
    }

    /// Asserts that the normalized (whitespace-free) code contains a certain substring
    fn assert_contains_ws_agnostic(haystack: &str, needle: &str, msg: &str) {
        let no_space = no_ws(haystack);
        let no_space_needle = no_ws(needle);
        assert!(no_space.contains(&no_space_needle), "{}\nNormalized code:\n{}", msg, no_space);
    }

    #[traced_test]
    fn test_non_optional_no_prefix_postfix() {
        info!("Starting test_non_optional_no_prefix_postfix...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                effect: MyEffect
            }
        };
        let field = extract_first_named_field(item_struct);

        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        // We'll do substring checks ignoring whitespace
        // We want to see prefix_space, postfix_space, self.effect.text(), features.push
        assert_contains_ws_agnostic(&expanded, "prefix_space", "Should define prefix_space");
        assert_contains_ws_agnostic(&expanded, "postfix_space", "Should define postfix_space");
        assert_contains_ws_agnostic(&expanded, "self.effect.text()", "Should call .text() on effect");
        assert_contains_ws_agnostic(&expanded, "features.push", "Should push the final string");

        info!("Finished test_non_optional_no_prefix_postfix successfully!");
    }

    #[traced_test]
    fn test_non_optional_prefix_postfix() {
        info!("Starting test_non_optional_prefix_postfix...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_prefix="PREFIX", feature_postfix="POSTFIX")]
                effect: MyEffect
            }
        };
        let field = extract_first_named_field(item_struct);

        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        // Should mention 'PREFIX', 'POSTFIX', do format!(..), call .text(), and push
        assert_contains_ws_agnostic(&expanded, "PREFIX", "Should insert 'PREFIX'");
        assert_contains_ws_agnostic(&expanded, "POSTFIX", "Should insert 'POSTFIX'");
        assert_contains_ws_agnostic(&expanded, "format!(", "Should have a format! call");
        assert_contains_ws_agnostic(&expanded, "self.effect.text()", "Should call .text()");
        assert_contains_ws_agnostic(&expanded, "features.push", "Should push the final string");

        info!("Finished test_non_optional_prefix_postfix successfully!");
    }

    #[traced_test]
    fn test_non_optional_with_feature_if_none_is_error() {
        info!("Starting test_non_optional_with_feature_if_none_is_error...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_if_none="Nope")]
                effect: MyEffect
            }
        };
        let field = extract_first_named_field(item_struct);

        let err = process_field(&field).unwrap_err().to_string();
        info!("Error was: {}", err);
        assert!(
            err.contains("only applicable to Option types"),
            "Should produce an error if feature_if_none is used on a non-Option field"
        );

        info!("Finished test_non_optional_with_feature_if_none_is_error successfully!");
    }

    #[traced_test]
    fn test_optional_no_feature_if_none() {
        info!("Starting test_optional_no_feature_if_none...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                side_effects: Option<SideEffects>
            }
        };
        let field = extract_first_named_field(item_struct);

        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        // Should do an if-let Some(value) approach plus prefix/postfix lines
        let nowhite = no_ws(&expanded);
        assert!(nowhite.contains("ifletSome(value)=&self.side_effects"),
            "Should do if-let for Some(...) block in:\n{}", nowhite);
        assert!(nowhite.contains("value.text()"), "Should call value.text()");
        assert!(nowhite.contains("prefix_space"), "Defines prefix_space");
        assert!(nowhite.contains("postfix_space"), "Defines postfix_space");
        // Should not contain "None=>" or match
        assert!(!nowhite.contains("None=>"), "No default branch without feature_if_none");
        assert!(!nowhite.contains("match&self.side_effects"), "Should not do match with no feature_if_none");

        info!("Finished test_optional_no_feature_if_none successfully!");
    }

    #[traced_test]
    fn test_optional_with_feature_if_none() {
        info!("Starting test_optional_with_feature_if_none...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_if_none="No side effects.")]
                side_effects: Option<SideEffects>
            }
        };
        let field = extract_first_named_field(item_struct);

        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        // We expect a match approach
        let nowhite = no_ws(&expanded);
        assert!(nowhite.contains("match&self.side_effects"), "Should match Some/None");
        assert!(nowhite.contains("Some(value)=>"), "Should handle Some(...)");
        assert!(nowhite.contains("None=>"), "Should handle None(...)");
        assert!(nowhite.contains("Nosideeffects."), "Should mention the default text");
        assert!(nowhite.contains("prefix_space"), "Should define prefix_space in arms");
        assert!(nowhite.contains("postfix_space"), "Should define postfix_space in arms");

        info!("Finished test_optional_with_feature_if_none successfully!");
    }

    #[traced_test]
    fn test_optional_with_feature_if_none_and_prefix_postfix() {
        info!("Starting test_optional_with_feature_if_none_and_prefix_postfix...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_if_none="No side effects.", feature_prefix="PREFIX", feature_postfix="POSTFIX")]
                side_effects: Option<SideEffects>
            }
        };
        let field = extract_first_named_field(item_struct);

        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        let nowhite = no_ws(&expanded);
        // check for match, Some(value), None, plus prefix/postfix references
        assert!(nowhite.contains("match&self.side_effects"), "Should do a match block");
        assert!(nowhite.contains("Some(value)=>"), "Some arm");
        assert!(nowhite.contains("None=>"), "None arm");
        assert!(nowhite.contains("PREFIX"), "prefix string");
        assert!(nowhite.contains("POSTFIX"), "postfix string");
        assert!(nowhite.contains("Nosideeffects."), "default text for None branch");

        info!("Finished test_optional_with_feature_if_none_and_prefix_postfix successfully!");
    }

    #[traced_test]
    fn test_non_optional_generic_example() {
        info!("Starting test_non_optional_generic_example...");
        let item_struct: ItemStruct = parse_quote! {
            struct TestStruct {
                field: i32
            }
        };

        let field = extract_first_named_field(item_struct);
        let expanded = process_field(&field).unwrap().to_string();
        info!("Expanded code:\n{}", expanded);

        // We'll see prefix_space, postfix_space, plus self.field.text()
        let nowhite = no_ws(&expanded);
        assert!(nowhite.contains("prefix_space"), "Should define prefix_space");
        assert!(nowhite.contains("postfix_space"), "Should define postfix_space");
        assert!(nowhite.contains("self.field.text()"), "Should call .text() on i32 field");
        assert!(nowhite.contains("features.push"), "Should push the final combined string");

        info!("Finished test_non_optional_generic_example successfully!");
    }
}
