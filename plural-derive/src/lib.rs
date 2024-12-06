use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit};

/// Procedural macro to derive `PluralDisplay`.
#[proc_macro_derive(Plural, attributes(plural))]
pub fn derive_plural(input: TokenStream) -> TokenStream {
    // Parse the input enum
    let input = parse_macro_input!(input as DeriveInput);

    // Ensure it's an enum
    let ident = input.ident.clone();
    let data = if let syn::Data::Enum(data) = input.data {
        data
    } else {
        return syn::Error::new_spanned(input, "Plural can only be derived for enums")
            .to_compile_error()
            .into();
    };

    if data.variants.is_empty() {
        // Handle empty enums by generating a dummy implementation
        return quote! {
            impl PluralDisplay for #ident {
                fn plural(&self) -> &'static str {
                    unreachable!("Empty enums cannot have instances.");
                }
            }
        }
        .into();
    }

    // Generate plural forms for each variant
    let variants = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        // Check for #[plural("...")] attribute
        let custom_plural = find_custom_plural(&variant.attrs);

        // Use custom plural if provided, else generate default
        let plural_form = if let Some(custom) = custom_plural {
            custom
        } else {
            // Convert CamelCase to lowercase with spaces and append 's'
            let name = variant_ident.to_string();
            let spaced = name
                .chars()
                .flat_map(|c| if c.is_uppercase() {
                    vec![' ', c.to_ascii_lowercase()]
                } else {
                    vec![c]
                })
                .collect::<String>()
                .trim()
                .to_owned();
            format!("{}s", spaced)
        };

        // Generate match arm
        quote! {
            Self::#variant_ident => #plural_form,
        }
    });

    // Implement the PluralDisplay trait
    let expanded = quote! {
        impl PluralDisplay for #ident {
            fn plural(&self) -> &'static str {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    expanded.into()
}

/// Extracts the custom plural value from the `#[plural("...")]` attribute, if present.
fn find_custom_plural(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("plural") {
            if let Some(Lit::Str(lit_str)) = attr.parse_args::<Lit>().ok() {
                return Some(lit_str.value());
            }
        }
    }
    None
}
