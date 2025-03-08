// ---------------- [ File: src/lib.rs ]
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
xp!{strip_surrounding_quotes}
xp!{try_parse_name_value}
xp!{try_parse_parenthesized}

#[proc_macro_derive(TokenExpansionAxis, attributes(system_message_goal, axis))]
pub fn derive_token_expander_axis(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let enum_ident = &ast.ident;
    let enum_name_str = enum_ident.to_string();

    // We’ll strip “ExpanderAxis” from the end of the enum name and prepend “Expanded”
    // for the data-carrying struct, and just “Expander” for the aggregator struct.
    // e.g.  ArtifactExpanderAxis -> ExpandedArtifact, ArtifactExpander
    let expanded_struct_name_str = if let Some(stripped) = enum_name_str.strip_suffix("ExpanderAxis") {
        format!("Expanded{}", stripped)
    } else {
        // fallback if not named “SomethingExpanderAxis”
        format!("Expanded{}", enum_name_str)
    };
    let aggregator_name_str = if let Some(stripped) = enum_name_str.strip_suffix("ExpanderAxis") {
        format!("{}Expander", stripped)
    } else {
        // fallback
        format!("{}Expander", enum_name_str)
    };

    let expanded_struct_ident = syn::Ident::new(&expanded_struct_name_str, enum_ident.span());
    let aggregator_ident      = syn::Ident::new(&aggregator_name_str, enum_ident.span());

    // Confirm it is an enum
    let data_enum = match &ast.data {
        syn::Data::Enum(enm) => enm,
        _ => {
            return syn::Error::new_spanned(enum_ident, "This macro only supports enums.")
                .to_compile_error()
                .into();
        }
    };

    // =========== PARSE system_message_goal string ===========
    let system_message_goal = parse_system_message_goal(&ast.attrs)
        .expect("Error parsing system_message_goal")
        .unwrap_or_else(|| syn::LitStr::new("Default system message goal", enum_ident.span()));

    // Prepare matches for AxisName / AxisDescription
    let mut axis_name_matches = Vec::new();
    let mut axis_desc_matches = Vec::new();

    // Collect variant idents for the aggregator’s axes() method
    let mut variant_idents = Vec::new();

    // We’ll also collect struct fields for each variant:
    //    field_name: Option<Vec<String>>
    let mut struct_fields = Vec::new();

    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        variant_idents.push(variant_ident);

        // Parse the #[axis("axis_name => axis_description")]
        let (axis_name, axis_desc) = variant
            .attrs
            .iter()
            .filter_map(|attr| parse_axis_attribute(attr).ok().flatten())
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "Missing #[axis(\"axis_name => axis_description\")] for variant {variant_ident}"
                );
            });

        axis_name_matches.push(quote! {
            #enum_ident::#variant_ident => Cow::Borrowed(#axis_name)
        });

        axis_desc_matches.push(quote! {
            #enum_ident::#variant_ident => Cow::Borrowed(#axis_desc)
        });

        // Generate a field for the expanded struct, based on `axis_name`
        let field_ident = syn::Ident::new(&axis_name, variant_ident.span());
        let field_ty: syn::Type = syn::parse_str("Option<Vec<String>>").unwrap();
        
        let field = quote! {
            #[serde(default)]
            #field_ident : #field_ty
        };
        struct_fields.push(field);
    }

    // 1) Generate `impl TokenExpansionAxis + AxisName + AxisDescription for <Enum>`
    //    (No longer implementing SystemMessageGoal here, so remove that.)
    //
    let expanded_impl_axis_traits = quote! {

        impl ::token_expander_axis::TokenExpansionAxis for #enum_ident { }

        impl ::token_expander_axis::AxisName for #enum_ident {
            fn axis_name(&self) -> std::borrow::Cow<'_,str> {
                match self {
                    #( #axis_name_matches ),*
                }
            }
        }

        impl ::token_expander_axis::AxisDescription for #enum_ident {
            fn axis_description(&self) -> std::borrow::Cow<'_,str> {
                match self {
                    #( #axis_desc_matches ),*
                }
            }
        }
    };

    // 2) Generate the data-carrying struct, e.g. `ExpandedArtifact`
    //
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

    // 2a) Generate LoadFromFile impl (if needed).
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
                    .map_err(|x| SaveLoadError::JsonParseError(x.into()))?;
                Ok(this)
            }
        }
    };

    // 3) Generate the aggregator struct, e.g. `ArtifactExpander`.
    //    This will implement `TokenExpander`, `SystemMessageGoal`, `Named`, etc.
    //
    let aggregator_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #aggregator_ident;

        impl Default for #aggregator_ident {
            fn default() -> Self { Self }
        }
    };

    // 3a) Implement Named for aggregator (if you have such a trait):
    let named_impl = quote! {
        impl Named for #aggregator_ident {
            fn name(&self) -> std::borrow::Cow<'_,str> {
                // You may choose something else here:
                std::borrow::Cow::Borrowed(#enum_name_str)
            }
        }
    };

    // 3b) Implement SystemMessageGoal for aggregator:
    let system_message_goal_impl = quote! {
        impl SystemMessageGoal for #aggregator_ident {
            fn system_message_goal(&self) -> std::borrow::Cow<'_,str> {
                std::borrow::Cow::Borrowed(#system_message_goal)
            }
        }
    };

    // 3c) Implement TokenExpander for aggregator:
    //     Return all variants of <Enum> in a Vec<Arc<dyn TokenExpansionAxis>>.
    let token_expander_impl = {

        let variant_arms = variant_idents.iter().map(|var_ident| {
            quote! {
                std::sync::Arc::new(#enum_ident::#var_ident)
            }
        });

        quote! {
            impl TokenExpander for #aggregator_ident {
                fn axes(&self) -> Vec<std::sync::Arc<dyn TokenExpansionAxis>> {
                    vec![
                        #( #variant_arms ),*
                    ]
                }
            }
        }
    };

    // Combine everything
    let expanded = quote! {
        // 1) Axis trait impls
        #expanded_impl_axis_traits

        // 2) Data struct + load impl
        #expanded_struct
        #load_from_file_impl

        // 3) Aggregator struct + trait impls
        #aggregator_struct
        #named_impl
        #system_message_goal_impl
        #token_expander_impl
    };

    TokenStream::from(expanded)
}
