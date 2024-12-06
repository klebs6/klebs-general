use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(VariantBuilder, attributes(default))]
pub fn variant_builder_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Ensure this is an enum
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("#[derive(VariantBuilder)] is only supported on enums"),
    };

    let mut default_variant = None;
    let mut builder_methods = Vec::new();

    // Iterate through the variants
    for variant in &data.variants {
        let variant_name = &variant.ident;

        // Check if the variant has the #[default] attribute
        if variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("default"))
        {
            if default_variant.is_some() {
                panic!("Only one variant can be marked as #[default]");
            }
            default_variant = Some(variant_name.clone());
        }

        // Convert the variant name to snake_case for the method name
        let builder_name = format_ident!("{}", variant_name.to_string().to_case(Case::Snake));

        // Generate the builder method for this variant
        let builder_code = match &variant.fields {
            Fields::Unnamed(fields) => {
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                if field_types.len() != 1 {
                    panic!("Each variant must have exactly one unnamed field.");
                }
                let field_type = &field_types[0];
                let builder_type = format_ident!("{}Builder", quote!(#field_type).to_string());
                quote! {
                    pub fn #builder_name<F>(build: F) -> Self
                where
                        F: FnOnce(&mut #builder_type),
                    {
                        let mut builder = #builder_type::default();
                        build(&mut builder);
                        Self::#variant_name(builder.build().expect("Builder failed to construct variant"))
                    }
                }
            }
            _ => panic!("Only unnamed fields are supported for variants."),
        };

        builder_methods.push(builder_code);
    }

    // Generate the Default impl if a default variant was found
    let default_impl = if let Some(default_variant) = default_variant {
        quote! {
            impl Default for #name {
                fn default() -> Self {
                    Self::#default_variant(Default::default())
                }
            }
        }
    } else {
        proc_macro2::TokenStream::new()
    };

    // Generate the final code
    let expanded = quote! {
        impl #name {
            #(#builder_methods)*
        }

        #default_impl
    };

    TokenStream::from(expanded)
}

