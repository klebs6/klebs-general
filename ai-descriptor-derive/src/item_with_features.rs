// ---------------- [ File: ai-descriptor-derive/src/item_with_features.rs ]
crate::ix!();

pub(crate) fn impl_item_with_features(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    use tracing::{trace, debug, info, warn, error};

    trace!("Entering impl_item_with_features for type '{}'", input.ident);

    let implement_display = crate::has_ai_display(&input.attrs);
    let name = &input.ident;

    // If we see #[ai(Display)] on the type, generate a Display impl using ai().
    let display_impl = if implement_display {
        trace!("'{}' has #[ai(Display)], generating Display impl", name);
        quote::quote! {
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.ai())
                }
            }
        }
    } else {
        quote::quote! {}
    };

    match &input.data {
        // ------------------------------------------------------------------
        // 1) Struct with named fields + #[ai("...")] at the type level
        // ------------------------------------------------------------------
        syn::Data::Struct(data_struct) => {
            trace!("'{}' is a struct; extracting header from #[ai(...)].", name);

            let header = match crate::find_ai_attr(&input.attrs) {
                Some(text) => text,
                None => {
                    error!("Struct '{}' is missing required #[ai(\"...\")] attribute for the header.", name);
                    return syn::Error::new_spanned(
                        input.ident.clone(),
                        "Structs deriving ItemWithFeatures must have an #[ai(\"...\")] attribute for the header",
                    )
                    .to_compile_error()
                    .into();
                }
            };

            let mut feature_expressions = Vec::new();
            match &data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    for field in &fields_named.named {
                        match crate::process_field(field) {
                            Ok(expr) => {
                                debug!("Generated feature expression for field '{:?}' in '{}'", field.ident, name);
                                feature_expressions.push(expr);
                            }
                            Err(err) => {
                                error!("Error while processing field '{:?}' in '{}'", field.ident, name);
                                return err.into();
                            }
                        }
                    }
                }
                _ => {
                    warn!("Struct '{}' does not have named fields. ItemWithFeatures requires named fields.", name);
                    return syn::Error::new_spanned(
                        input.ident.clone(),
                        "ItemWithFeatures can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }

            trace!("Successfully generated feature expressions for struct '{}'", name);

            let expanded = quote::quote! {
                impl ItemWithFeatures for #name {
                    fn header(&self) -> std::borrow::Cow<'_, str> {
                        std::borrow::Cow::Borrowed(#header)
                    }

                    fn features(&self) -> Vec<std::borrow::Cow<'_, str>> {
                        use std::borrow::Cow;
                        let mut features = Vec::new();
                        #(#feature_expressions;)*
                        features
                    }
                }

                #display_impl
            };

            info!("Finished generating ItemWithFeatures for struct '{}'", name);
            expanded
        }

        // ------------------------------------------------------------------
        // 2) Enums: each variant must have exactly one unnamed field.
        //
        //    - If the variant has #[ai(wrap="ItemWithFeatures")], then we treat
        //      the single field as T: ItemWithFeatures.
        //
        //        * The variant can optionally have:
        //             #[ai(header_prefix="...")] and/or #[ai(header_postfix="...")]
        //             plus an optional string literal descriptor.
        //          We'll combine them around the inner.header() as follows:
        //             final_header = prefix + [ descriptor_if_any : inner_header ] + postfix
        //
        //        * The final features come from inner.features().
        //
        //    - If the variant does NOT have wrap="ItemWithFeatures", we treat
        //      the single field as T: ItemFeature. In that case:
        //
        //        * We REQUIRE a string literal descriptor for the variant’s header.
        //        * The variant’s features are just `[ inner.text() ]`.
        //        * (prefix/postfix are allowed, but you'd still need a descriptor for
        //          the main header content.)
        // ------------------------------------------------------------------
        syn::Data::Enum(data_enum) => {
            trace!("'{}' is an enum; generating match arms for each variant.", name);

            #[derive(Default,Debug, Clone, Copy)]
            enum WrapKind {
                ItemWithFeatures,

                #[default]
                ItemFeature,
            }

            // Holds all data we might parse from #[ai(...)]
            #[derive(Debug, Default)]
            struct VariantAiInfo {
                wrap_kind: WrapKind,               // defaults to ItemFeature
                descriptor: Option<String>,        // the literal string in #[ai("...")]
                header_prefix: Option<String>,     // #[ai(header_prefix="...")]
                header_postfix: Option<String>,    // #[ai(header_postfix="...")]
            }

            impl VariantAiInfo {
                fn new() -> Self {
                    VariantAiInfo {
                        wrap_kind: WrapKind::ItemFeature,
                        descriptor: None,
                        header_prefix: None,
                        header_postfix: None,
                    }
                }
            }

            /// Parse the variant-level `#[ai(...)]` attributes. We look for:
            ///  - optional wrap="ItemWithFeatures"
            ///  - optional string literal descriptor
            ///  - optional header_prefix="..."
            ///  - optional header_postfix="..."
            fn parse_variant_ai_attributes(
                variant_ident: &syn::Ident,
                attrs: &[syn::Attribute],
            ) -> syn::Result<VariantAiInfo> {
                let mut info = VariantAiInfo::new();

                for attr in attrs {
                    if attr.path.is_ident("ai") {
                        if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
                            for nested in &meta_list.nested {
                                match nested {
                                    // e.g. #[ai("some descriptor")]
                                    syn::NestedMeta::Lit(syn::Lit::Str(lit_str)) => {
                                        if info.descriptor.is_some() {
                                            return Err(syn::Error::new_spanned(
                                                lit_str,
                                                "Multiple string literals found in #[ai(...)]. Only one is allowed.",
                                            ));
                                        }
                                        info.descriptor = Some(lit_str.value());
                                    }

                                    // e.g. #[ai(wrap="ItemWithFeatures")]
                                    syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                                        if nv.path.is_ident("wrap") {
                                            if let syn::Lit::Str(ref val) = nv.lit {
                                                if val.value() == "ItemWithFeatures" {
                                                    info.wrap_kind = WrapKind::ItemWithFeatures;
                                                } else {
                                                    return Err(syn::Error::new_spanned(
                                                        &nv.lit,
                                                        format!(
                                                            "Unknown wrap kind '{}'. Only 'ItemWithFeatures' is valid.",
                                                            val.value()
                                                        ),
                                                    ));
                                                }
                                            } else {
                                                return Err(syn::Error::new_spanned(
                                                    &nv.lit,
                                                    "wrap= must be a string literal",
                                                ));
                                            }
                                        } else if nv.path.is_ident("header_prefix") {
                                            if let syn::Lit::Str(ref prefix_val) = nv.lit {
                                                info.header_prefix = Some(prefix_val.value());
                                            } else {
                                                return Err(syn::Error::new_spanned(
                                                    &nv.lit,
                                                    "header_prefix= must be a string literal",
                                                ));
                                            }
                                        } else if nv.path.is_ident("header_postfix") {
                                            if let syn::Lit::Str(ref postfix_val) = nv.lit {
                                                info.header_postfix = Some(postfix_val.value());
                                            } else {
                                                return Err(syn::Error::new_spanned(
                                                    &nv.lit,
                                                    "header_postfix= must be a string literal",
                                                ));
                                            }
                                        } else {
                                            return Err(syn::Error::new_spanned(
                                                &nv.path,
                                                "Unsupported key in #[ai(...)] on an enum variant",
                                            ));
                                        }
                                    }

                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            nested,
                                            "Unsupported syntax in #[ai(...)]. \
                                             Expected a string literal, wrap=\"...\", header_prefix=\"...\", or header_postfix=\"...\"",
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                // No immediate error if there's no descriptor. We'll decide logic below.
                Ok(info)
            }

            let mut header_arms = Vec::new();
            let mut features_arms = Vec::new();

            for variant in &data_enum.variants {
                let variant_ident = &variant.ident;

                // Each variant must be exactly one unnamed field
                let fields_unnamed = match &variant.fields {
                    syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => &u.unnamed,
                    _ => {
                        error!(
                            "Enum variant '{}::{}' does not have exactly one unnamed field.",
                            name, variant_ident
                        );
                        return syn::Error::new_spanned(
                            variant_ident.clone(),
                            "Enum variants deriving ItemWithFeatures must have exactly one unnamed field",
                        )
                        .to_compile_error()
                        .into();
                    }
                };

                // Parse the variant's #[ai(...)] attribute
                let info = match parse_variant_ai_attributes(variant_ident, &variant.attrs) {
                    Ok(vals) => vals,
                    Err(e) => return e.to_compile_error().into(),
                };

                // We'll define some helpers for prefix/postfix to avoid repeated code
                let prefix = info.header_prefix.unwrap_or_default();
                let postfix = info.header_postfix.unwrap_or_default();

                match info.wrap_kind {
                    //
                    // ==============================
                    // wrap="ItemWithFeatures"
                    // ==============================
                    WrapKind::ItemWithFeatures => {
                        // The single field is T: ItemWithFeatures.
                        // We may or may not have an outer descriptor.
                        // We'll always produce:
                        //   let inner_header = inner.header().to_string();
                        //   if descriptor.is_some():
                        //       final_header = prefix + ( descriptor + ":" + inner_header ) + postfix
                        //   else:
                        //       final_header = prefix + inner_header + postfix
                        //
                        let arm_header = if let Some(ref outer_desc) = info.descriptor {
                            quote::quote! {
                                let inner_header = ItemWithFeatures::header(inner).to_string();
                                let combined = format!("{}{} {}{}", #prefix, #outer_desc, inner_header, #postfix);
                                std::borrow::Cow::Owned(combined)
                            }
                        } else {
                            quote::quote! {
                                let inner_header = ItemWithFeatures::header(inner);
                                if #prefix.is_empty() && #postfix.is_empty() {
                                    // zero overhead, just pass it along
                                    inner_header
                                } else {
                                    // convert to string for safe concatenation
                                    let ih = inner_header.to_string();
                                    let combined = format!("{}{} {}", #prefix, ih, #postfix);
                                    std::borrow::Cow::Owned(combined)
                                }
                            }
                        };

                        header_arms.push(quote::quote! {
                            #name::#variant_ident(inner) => {
                                #arm_header
                            }
                        });

                        // Features: just the inner's features
                        features_arms.push(quote::quote! {
                            #name::#variant_ident(inner) => {
                                ItemWithFeatures::features(inner)
                            }
                        });
                    }

                    //
                    // ==============================
                    // wrap="ItemFeature" (default)
                    // ==============================
                    WrapKind::ItemFeature => {
                        // The single field is T: ItemFeature, not ItemWithFeatures.
                        // We REQUIRE a descriptor for the final header text. If none is present => error.
                        if info.descriptor.is_none() {
                            let err_msg = format!(
                                "Enum variant '{}::{}' wraps an ItemFeature but has no #[ai(\"...\")] descriptor. \
                                 Provide a string literal or switch to #[ai(wrap=\"ItemWithFeatures\")].",
                                name, variant_ident
                            );
                            error!("{}", err_msg);
                            return syn::Error::new_spanned(
                                variant_ident,
                                err_msg
                            )
                            .to_compile_error()
                            .into();
                        }

                        let desc = info.descriptor.unwrap();
                        // We'll combine prefix + desc + postfix for the final header.
                        let arm_header = quote::quote! {
                            let combined = format!("{}{}{}", #prefix, #desc, #postfix);
                            std::borrow::Cow::Owned(combined)
                        };

                        header_arms.push(quote::quote! {
                            #name::#variant_ident(_inner) => {
                                #arm_header
                            }
                        });

                        // For features, we push a single line = inner.text().
                        features_arms.push(quote::quote! {
                            #name::#variant_ident(inner) => {
                                let mut f = Vec::new();
                                f.push(ItemFeature::text(inner));
                                f
                            }
                        });
                    }
                }
            }

            // Build the final enum impl
            let expanded = quote::quote! {
                impl ItemWithFeatures for #name {
                    fn header(&self) -> std::borrow::Cow<'_, str> {
                        match self {
                            #(#header_arms),*
                        }
                    }

                    fn features(&self) -> Vec<std::borrow::Cow<'_, str>> {
                        match self {
                            #(#features_arms),*
                        }
                    }
                }

                #display_impl
            };

            info!("Finished generating ItemWithFeatures for enum '{}'", name);
            expanded
        }

        // ------------------------------------------------------------------
        // Otherwise, we cannot derive
        // ------------------------------------------------------------------
        _ => {
            error!(
                "Type '{}' is neither a struct-with-named-fields nor an enum with single-unnamed-field variants",
                name
            );
            syn::Error::new_spanned(
                input.ident.clone(),
                "ItemWithFeatures can only be derived for structs with named fields \
                 (with a #[ai(\"...\")] for the header), or for enums whose variants each \
                 have exactly one unnamed field plus optional #[ai(...)] attributes.",
            )
            .to_compile_error()
            .into()
        }
    }
}
