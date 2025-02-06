use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Variant, LitStr};

#[proc_macro_derive(FileDownloader, attributes(download_link))]
pub fn derive_file_downloader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let download_link_body = match &input.data {
        Data::Enum(data_enum) => {
            let arms = data_enum.variants.iter().map(|variant| {
                variant_match_arm(variant)
            });
            quote! {
                match self {
                    #(#arms)*
                }
            }
        },
        _ => {
            return syn::Error::new_spanned(
                ident,
                "FileDownloader can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {

        impl #impl_generics DownloadLink for #ident #ty_generics #where_clause {
            fn download_link(&self) -> &str {
                #download_link_body
            }
        }

        impl #impl_generics Md5DownloadLink for #ident #ty_generics #where_clause {}

        impl #impl_generics FileDownloader for #ident #ty_generics #where_clause {}

    };

    expanded.into()
}

fn variant_match_arm(variant: &Variant) -> proc_macro2::TokenStream {
    let var_ident = &variant.ident;
    let mut download_link_attr: Option<String> = None;

    for attr in &variant.attrs {
        if attr.path().is_ident("download_link") {
            // Attempt to parse a string from the attribute
            if let Ok(lit_str) = attr.parse_args::<LitStr>() {
                download_link_attr = Some(lit_str.value());
            }
        }
    }

    match &variant.fields {
        syn::Fields::Unit => {
            if let Some(dl) = download_link_attr {
                // <-- KEY FIX: produce a *literal* expression
                let link_lit = proc_macro2::Literal::string(&dl);
                quote! {
                    Self::#var_ident => #link_lit,
                }
            } else {
                syn::Error::new_spanned(
                    variant,
                    "Missing `#[download_link = \"...\"]` attribute on this unit variant"
                )
                .to_compile_error()
            }
        },
        syn::Fields::Unnamed(fields) => {
            if let Some(dl) = download_link_attr {
                let link_lit = proc_macro2::Literal::string(&dl);
                quote! {
                    Self::#var_ident(..) => #link_lit,
                }
            } else {
                if fields.unnamed.len() == 1 {
                    quote! {
                        Self::#var_ident(inner) => inner.download_link(),
                    }
                } else {
                    syn::Error::new_spanned(
                        variant,
                        "Multiple fields in tuple variant not supported without a download_link"
                    )
                    .to_compile_error()
                }
            }
        },
        syn::Fields::Named(fields) => {
            if let Some(dl) = download_link_attr {
                let link_lit = proc_macro2::Literal::string(&dl);
                quote! {
                    Self::#var_ident {..} => #link_lit,
                }
            } else {
                if fields.named.len() == 1 {
                    let field_name = &fields.named.iter().next().unwrap().ident;
                    quote! {
                        Self::#var_ident { #field_name } => #field_name.download_link(),
                    }
                } else {
                    syn::Error::new_spanned(
                        variant,
                        "Multiple named fields not supported without a download_link"
                    )
                    .to_compile_error()
                }
            }
        }
    }
}
