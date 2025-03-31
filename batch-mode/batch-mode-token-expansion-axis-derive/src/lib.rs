// ---------------- [ File: batch-mode-token-expansion-axis-derive/src/lib.rs ]
//! The `token-expander-axis-derive` crate defines the custom `#[derive(TokenExpansionAxis)]`
//! procedural macro. It reads the enum, extracts the `#[system_message_goal("...")]`
//! and `#[axis("axis_name" => "axis_description")]` attributes, and implements
//! the `SystemMessageGoal`, `AxisName`, and `AxisDescription` traits from the
//! `token-expander-axis` crate.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
//#![deny(missing_docs)]

extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{system_message_goal}
xp!{axis_attribute}
xp!{try_parse_name_value}
xp!{try_parse_parenthesized}

#[proc_macro_derive(TokenExpansionAxis, attributes(system_message_goal, axis))]
pub fn derive_token_expander_axis(input: TokenStream) -> TokenStream {
    use syn::spanned::Spanned;

    trace!("Parsing input for TokenExpansionAxis derive macro.");
    let ast = parse_macro_input!(input as DeriveInput);

    let enum_ident = &ast.ident;
    let enum_name_str = enum_ident.to_string();
    debug!("Found enum name: {}", enum_name_str);

    // We’ll strip “ExpanderAxis” from the end of the enum name and
    // create the expanded struct + aggregator with the usual naming strategy.
    let expanded_struct_name_str = if let Some(stripped) = enum_name_str.strip_suffix("ExpanderAxis") {
        format!("Expanded{}", stripped)
    } else {
        format!("Expanded{}", enum_name_str)
    };
    let aggregator_name_str = if let Some(stripped) = enum_name_str.strip_suffix("ExpanderAxis") {
        format!("{}Expander", stripped)
    } else {
        format!("{}Expander", enum_name_str)
    };

    let expanded_struct_ident = syn::Ident::new(&expanded_struct_name_str, enum_ident.span());
    let aggregator_ident = syn::Ident::new(&aggregator_name_str, enum_ident.span());

    // Ensure this derive only applies to enums
    let data_enum = match &ast.data {
        syn::Data::Enum(e) => {
            trace!("Confirmed that {} is an enum.", enum_name_str);
            e
        }
        _ => {
            let err = syn::Error::new_spanned(enum_ident, "This macro only supports enums.");
            error!("Encountered a non-enum type for #[derive(TokenExpansionAxis)]: {}", err);
            return err.to_compile_error().into();
        }
    };

    // Attempt to parse any system_message_goal attribute on the enum
    let system_message_goal = match parse_system_message_goal(&ast.attrs) {
        Ok(Some(lit_str)) => {
            debug!("system_message_goal attribute found: {:?}", lit_str.value());
            lit_str
        }
        Ok(None) => {
            debug!("No system_message_goal found; using default fallback.");
            syn::LitStr::new("Default system message goal", enum_ident.span())
        }
        Err(err) => {
            error!("Error parsing system_message_goal: {}", err);
            return err.to_compile_error().into();
        }
    };

    // Gather data for implementing AxisName/AxisDescription,
    // plus building the aggregator’s axes() and the struct fields.
    let mut axis_name_matches = Vec::new();
    let mut axis_desc_matches = Vec::new();
    let mut variant_idents = Vec::new();
    let mut struct_fields = Vec::new();

    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        trace!("Processing variant: {}", variant_ident);

        variant_idents.push(variant_ident);

        // We expect each variant to have exactly one #[axis("... => ...")]
        let (axis_name, axis_desc) = variant
            .attrs
            .iter()
            .filter_map(|attr| parse_axis_attribute(attr).ok().flatten())
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "Missing #[axis(\"axis_name => axis_description\")] on variant '{}'",
                    variant_ident
                );
            });

        // For AxisName / AxisDescription matching:
        axis_name_matches.push(quote! {
            #enum_ident::#variant_ident => std::borrow::Cow::Borrowed(#axis_name)
        });
        axis_desc_matches.push(quote! {
            #enum_ident::#variant_ident => std::borrow::Cow::Borrowed(#axis_desc)
        });

        // Generate a field in the expanded struct (snake_case from the axis name)
        let field_name_str = axis_name.to_snake_case();
        let field_ident = syn::Ident::new(&field_name_str, variant_ident.span());
        let field_ty: syn::Type = syn::parse_str("Option<Vec<String>>").unwrap();

        let field = quote! {
            #[serde(default)]
            #field_ident : #field_ty
        };
        struct_fields.push(field);
    }

    // 1) Implement the axis traits on the original enum
    let expanded_impl_axis_traits = quote! {
        impl TokenExpansionAxis for #enum_ident {}

        impl AxisName for #enum_ident {
            fn axis_name(&self) -> std::borrow::Cow<'_, str> {
                match self {
                    #( #axis_name_matches ),*
                }
            }
        }

        impl AxisDescription for #enum_ident {
            fn axis_description(&self) -> std::borrow::Cow<'_, str> {
                match self {
                    #( #axis_desc_matches ),*
                }
            }
        }
    };

    // 2) The data-carrying struct: Expanded{Enum}
    let expanded_struct = quote! {
        #[derive(getset::Getters, Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[getset(get="pub")]
        pub struct #expanded_struct_ident {
            #[serde(alias = "token_name")]
            name: String,
            #( #struct_fields ),*
        }

        unsafe impl Send for #expanded_struct_ident {}
        unsafe impl Sync for #expanded_struct_ident {}

        #[async_trait::async_trait]
        impl ExpandedToken for #expanded_struct_ident {
            type Expander = #aggregator_ident;
        }
    };

    // 2a) Implement LoadFromFile if needed
    let load_from_file_impl = quote! {
        #[async_trait::async_trait]
        impl LoadFromFile for #expanded_struct_ident {
            type Error = SaveLoadError;

            async fn load_from_file(filename: impl AsRef<std::path::Path> + Send)
                -> Result<Self, Self::Error>
            {
                info!("loading token expansion from file {:?}", filename.as_ref());
                let json = std::fs::read_to_string(filename)?;
                let this = serde_json::from_str(&json)
                    .map_err(|e| SaveLoadError::JsonParseError(e.into()))?;
                Ok(this)
            }
        }
    };

    // 3) The aggregator struct: e.g. EnumExpander
    let aggregator_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #aggregator_ident;

        impl Default for #aggregator_ident {
            fn default() -> Self {
                Self
            }
        }
    };

    // 3a) Named for aggregator => return the aggregator’s own name
    let named_impl = quote! {
        impl Named for #aggregator_ident {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                // IMPORTANT: We actually want the aggregator name here,
                // which is aggregator_name_str, not the enum's name.
                std::borrow::Cow::Borrowed(#aggregator_name_str)
            }
        }
    };

    // 3b) SystemMessageGoal => use the parsed or default system_message_goal
    let system_message_goal_impl = quote! {
        impl SystemMessageGoal for #aggregator_ident {
            fn system_message_goal(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed(#system_message_goal)
            }
        }
    };

    // 3c) TokenExpander + GetTokenExpansionAxes => aggregator’s axes
    let variant_arms = variant_idents.iter().map(|v| {
        quote! { std::sync::Arc::new(#enum_ident::#v) }
    });

    let token_expander_impl = quote! {
        impl TokenExpander for #aggregator_ident {}

        impl GetTokenExpansionAxes for #aggregator_ident {
            fn axes(&self) -> TokenExpansionAxes {
                vec![
                    #( #variant_arms ),*
                ]
            }
        }
    };

    // Combine everything
    let expanded = quote! {
        use batch_mode_token_expansion_traits::*;

        #expanded_impl_axis_traits
        #expanded_struct
        #load_from_file_impl

        #aggregator_struct
        #named_impl
        #system_message_goal_impl
        #token_expander_impl
    };

    trace!("Macro expansion for {} completed.", enum_name_str);
    TokenStream::from(expanded)
}
