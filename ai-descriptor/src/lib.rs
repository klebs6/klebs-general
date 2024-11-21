extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Lit, Meta, MetaList, MetaNameValue, NestedMeta,
};

#[proc_macro_derive(AIDescriptor, attributes(ai))]
pub fn derive_ai_descriptor(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_ast = parse_macro_input!(input as DeriveInput);

    // Get the name of the type (enum or struct)
    let type_name = &input_ast.ident;

    // Check if the input is an enum or struct
    match &input_ast.data {
        Data::Enum(data_enum) => {
            // Handle enums
            derive_ai_descriptor_for_enum(type_name, data_enum)
        }
        Data::Struct(data_struct) => {
            // Handle structs
            derive_ai_descriptor_for_struct(type_name, data_struct, &input_ast.attrs)
        }
        _ => {
            syn::Error::new_spanned(
                &input_ast.ident,
                "AIDescriptor can only be derived for enums and structs",
            )
            .to_compile_error()
            .into()
        }
    }
}

// Function to handle enums
fn derive_ai_descriptor_for_enum(
    enum_name: &syn::Ident,
    data_enum: &syn::DataEnum,
) -> TokenStream {
    let variants = &data_enum.variants;

    let mut variant_arms = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;
        let mut ai_description = None;

        // Look for #[ai("...")] or #[ai = "..."] attribute
        for attr in &variant.attrs {
            if attr.path.is_ident("ai") {
                let meta = attr.parse_meta().unwrap();
                match meta {
                    Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. }) => {
                        ai_description = Some(lit_str.value());
                    },
                    Meta::List(MetaList { nested, .. }) => {
                        for nested_meta in nested {
                            if let NestedMeta::Lit(Lit::Str(lit_str)) = nested_meta {
                                ai_description = Some(lit_str.value());
                            } else {
                                return syn::Error::new_spanned(
                                    nested_meta,
                                    "Expected string literal in #[ai(\"...\")]",
                                )
                                .to_compile_error()
                                .into();
                            }
                        }
                    },
                    _ => {
                        return syn::Error::new_spanned(
                            meta,
                            "Invalid format for #[ai(\"...\")]",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
        }

        if let Some(description) = ai_description {
            variant_arms.push(quote! {
                #enum_name::#variant_name => std::borrow::Cow::Borrowed(#description),
            });
        } else {
            return syn::Error::new_spanned(
                variant_name,
                format!("Variant `{}` is missing #[ai(\"...\")] attribute", variant_name),
            )
            .to_compile_error()
            .into();
        }
    }

    // Generate the impl block
    let gen = quote! {
        impl AIDescriptor for #enum_name {
            fn ai(&self) -> std::borrow::Cow<'_, str> {
                match self {
                    #(#variant_arms)*
                }
            }
        }
    };

    gen.into()
}

// Function to handle structs
fn derive_ai_descriptor_for_struct(
    struct_name: &syn::Ident,
    data_struct: &syn::DataStruct,
    attrs: &[syn::Attribute],
) -> TokenStream {
    // Extract the ai attribute from the struct
    let mut struct_ai_template = None;

    for attr in attrs {
        if attr.path.is_ident("ai") {
            let meta = attr.parse_meta().unwrap();
            match meta {
                Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. }) => {
                    struct_ai_template = Some(lit_str.value());
                },
                Meta::List(MetaList { nested, .. }) => {
                    for nested_meta in nested {
                        if let NestedMeta::Lit(Lit::Str(lit_str)) = nested_meta {
                            struct_ai_template = Some(lit_str.value());
                        } else {
                            return syn::Error::new_spanned(
                                nested_meta,
                                "Expected string literal in #[ai(\"...\")]",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                },
                _ => {
                    return syn::Error::new_spanned(
                        meta,
                        "Invalid format for #[ai(\"...\")]",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    let struct_ai_template = struct_ai_template.unwrap_or_default();

    let fields = match &data_struct.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    let mut field_bindings = Vec::new();
    let mut format_args = Vec::new();

    for field in &fields {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| syn::Ident::new("unnamed", field.span()));

        let field_name_str = field_name.to_string();

        if struct_ai_template.contains(&format!("{{{}}}", field_name_str)) {
            field_bindings.push(quote! {
                let #field_name = self.#field_name.ai();
            });
            format_args.push(quote! {
                #field_name = #field_name
            });
        }
    }

    let gen = quote! {
        impl AIDescriptor for #struct_name {
            fn ai(&self) -> std::borrow::Cow<'_, str> {
                #(#field_bindings)*

                let description = format!(#struct_ai_template, #(#format_args),*);
                std::borrow::Cow::Owned(description)
            }
        }
    };

    gen.into()
}

