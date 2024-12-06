use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(VariantBuilder)]
pub fn variant_builder_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;

    let variants = if let Data::Enum(data_enum) = input.data {
        data_enum.variants
    } else {
        panic!("#[derive(VariantBuilder)] is only applicable to enums.");
    };

    let methods = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Convert the variant name to snake_case for the method name
        let method_name = Ident::new(
            &variant_name.to_string().to_case(Case::Snake),
            variant_name.span(),
        );

        let builder_type = if let Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() != 1 {
                panic!("Each variant must have exactly one unnamed field.");
            }
            &fields.unnamed.first().unwrap().ty
        } else {
            panic!("Each variant must have a single unnamed field.");
        };

        let builder_type_name = if let syn::Type::Path(type_path) = builder_type {
            let last_segment = &type_path.path.segments.last().unwrap();
            let type_ident = &last_segment.ident;
            Ident::new(&format!("{}Builder", type_ident), type_ident.span())
        } else {
            panic!("Variant must contain a named builder type.");
        };

        quote! {
            pub fn #method_name<F>(build: F) -> Self
            where
                F: FnOnce(&mut #builder_type_name),
            {
                let mut builder = #builder_type_name::default();
                build(&mut builder);
                Self::#variant_name(builder.build().unwrap())
            }
        }
    });

    let expanded = quote! {
        impl #enum_name {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}

