// ---------------- [ File: ai-descriptor-derive/src/impl_item_feature_enum.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub(crate) fn impl_item_feature_enum(input: &syn::DeriveInput, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    use tracing::{trace, debug, info, warn, error};
    trace!("Entering impl_item_feature_enum for enum '{}'", input.ident);

    let enum_name = &input.ident;

    // A "leaf-enum" is one in which *all* variants are unit variants (no fields at all).
    let is_leaf_enum = data.variants.iter().all(|v| matches!(v.fields, syn::Fields::Unit));
    debug!("is_leaf_enum = {}", is_leaf_enum);

    // Parse optional top-level #[ai(feature_prefix="...")] / #[ai(feature_postfix="...")] attributes.
    // We only apply these if is_leaf_enum = true.
    #[derive(Default)]
    struct EnumOuterAi {
        feature_prefix: String,
        feature_postfix: String,
    }

    fn parse_enum_outer_ai(attrs: &[syn::Attribute]) -> EnumOuterAi {
        let mut result = EnumOuterAi::default();
        for attr in attrs {
            if attr.path.is_ident("ai") {
                if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested.iter() {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("feature_prefix") {
                                if let syn::Lit::Str(ref val) = nv.lit {
                                    result.feature_prefix = val.value();
                                }
                            } else if nv.path.is_ident("feature_postfix") {
                                if let syn::Lit::Str(ref val) = nv.lit {
                                    result.feature_postfix = val.value();
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }

    let outer_ai = if is_leaf_enum {
        parse_enum_outer_ai(&input.attrs)
    } else {
        EnumOuterAi::default()
    };
    debug!("Parsed enum-level feature_prefix='{}', feature_postfix='{}'",
           outer_ai.feature_prefix, outer_ai.feature_postfix);

    // If we *aren't* a leaf-enum, fall back to the existing logic that calls process_variant().
    // Otherwise, we manually build the match arms so we can wrap the variant text with prefix/postfix.
    if !is_leaf_enum {
        trace!("Enum '{}' is not a leaf-enum (some variants have fields). Using standard process_variant approach.", enum_name);

        let mut variant_matches = vec![];
        for variant in &data.variants {
            match crate::process_variant(variant) {
                Ok(variant_match) => variant_matches.push(variant_match),
                Err(error_ts) => {
                    error!("Error while processing variant '{}': returning error tokens", variant.ident);
                    return error_ts;
                }
            }
        }

        let expanded = quote::quote! {
            impl ItemFeature for #enum_name {
                fn text(&self) -> std::borrow::Cow<'_, str> {
                    match self {
                        #(#variant_matches)*
                    }
                }
            }
        };
        info!("Finished impl_item_feature_enum for non-leaf enum '{}'", enum_name);
        return expanded;
    } else {
        // Here, every variant *must* be unit, and we allow a #[ai("...")] on each variant for text.
        // We also apply top-level prefix/postfix if given.
        trace!("Enum '{}' is a leaf-enum; applying top-level feature_prefix/postfix to each variant's text.", enum_name);

        let prefix_str = &outer_ai.feature_prefix;
        let postfix_str = &outer_ai.feature_postfix;

        let mut arms = vec![];
        for variant in &data.variants {
            let variant_ident = &variant.ident;
            // For a unit variant, we *require* #[ai("some text")]
            let maybe_ai_text = crate::find_ai_attr(&variant.attrs);
            let ai_text = match maybe_ai_text {
                Some(t) => t,
                None => {
                    let msg = format!(
                        "Unit variant '{}::{}' is missing required #[ai(\"...\")] attribute",
                        enum_name, variant_ident
                    );
                    error!("{}", msg);
                    let compile_err = syn::Error::new_spanned(variant_ident, msg)
                        .to_compile_error();
                    return compile_err;
                }
            };

            arms.push(quote::quote! {
                Self::#variant_ident => {
                    let prefix_space = if #prefix_str.is_empty() {
                        String::new()
                    } else {
                        format!("{} ", #prefix_str)
                    };
                    let postfix_space = if #postfix_str.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", #postfix_str)
                    };

                    if #prefix_str.is_empty() && #postfix_str.is_empty() {
                        // No prefix/postfix => return Cow::Borrowed directly
                        std::borrow::Cow::Borrowed(#ai_text)
                    } else {
                        // Return Cow::Owned, combining prefix + text + postfix
                        std::borrow::Cow::Owned(format!("{}{}{}", prefix_space, #ai_text, postfix_space))
                    }
                }
            });
        }

        let expanded = quote::quote! {
            impl ItemFeature for #enum_name {
                fn text(&self) -> std::borrow::Cow<'_, str> {
                    match self {
                        #(#arms),*
                    }
                }
            }
        };
        info!("Finished impl_item_feature_enum for leaf-enum '{}'", enum_name);
        expanded
    }
}
