use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(VariantBuilder, attributes(default, no_builder, nested_builder))]
pub fn variant_builder_macro(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use tracing::trace;

    // ──────────────────────────────────────────────────────────────
    // Parse and validate input
    // ──────────────────────────────────────────────────────────────
    let input     = syn::parse_macro_input!(input as syn::DeriveInput);
    let enum_ident = input.ident.clone();

    trace!("Generating VariantBuilder impl for enum `{}`", enum_ident);

    let data = match input.data {
        syn::Data::Enum(d) => d,
        _ => panic!("#[derive(VariantBuilder)] can only be applied to enums"),
    };

    // ──────────────────────────────────────────────────────────────
    // Build helper methods
    // ──────────────────────────────────────────────────────────────
    let mut default_variant: Option<syn::Ident> = None;
    let mut helper_fns = Vec::<proc_macro2::TokenStream>::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        // Every variant must be a 1‑tuple variant.
        let field_ty = match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!(
                "Variant `{}` must be a tuple variant with exactly one unnamed field.",
                variant_ident
            ),
        };

        // detect #[default]
        if variant.attrs.iter().any(|a| a.path().is_ident("default")) {
            if default_variant.is_some() {
                panic!("Only one variant in `{}` can be marked #[default]", enum_ident);
            }
            default_variant = Some(variant_ident.clone());
        }

        let has_no_builder = variant
            .attrs
            .iter()
            .any(|a| a.path().is_ident("no_builder"));

        let has_nested_builder = variant
            .attrs
            .iter()
            .any(|a| a.path().is_ident("nested_builder"));

        if has_no_builder && has_nested_builder {
            panic!(
                "Variant `{}` cannot use both #[no_builder] and #[nested_builder]",
                variant_ident
            );
        }

        let helper_fn_ident = quote::format_ident!(
            "{}",
            variant_ident.to_string().to_case(convert_case::Case::Snake)
        );

        trace!(
            variant = %variant_ident,
            helper  = %helper_fn_ident,
            no_builder = has_no_builder,
            nested_builder = has_nested_builder,
            "emitting helper"
        );

        if has_no_builder {
            // direct‑value mode
            helper_fns.push(quote::quote! {
                pub fn #helper_fn_ident(inner: #field_ty) -> Self {
                    Self::#variant_ident(inner)
                }
            });
        } else if has_nested_builder {
            // nested‑builder mode
            helper_fns.push(quote::quote! {
                pub fn #helper_fn_ident<F>(build: F) -> Self
                where
                    F: FnOnce() -> #field_ty,
                {
                    Self::#variant_ident(build())
                }
            });
        } else {
            // legacy builder‑delegation mode
            let builder_ty_str = format!("{}Builder", quote::quote!(#field_ty));
            let builder_ty: syn::Type =
                syn::parse_str(&builder_ty_str).expect("valid builder type path");

            helper_fns.push(quote::quote! {
                pub fn #helper_fn_ident<F>(build: F) -> Self
                where
                    F: FnOnce(&mut #builder_ty),
                {
                    let mut builder = #builder_ty::default();
                    build(&mut builder);
                    Self::#variant_ident(
                        builder.build().expect("Builder failed to construct inner value")
                    )
                }
            });
        }
    }

    // ──────────────────────────────────────────────────────────────
    // Optional Default impl
    // ──────────────────────────────────────────────────────────────
    let default_impl = if let Some(def_ident) = default_variant {
        quote::quote! {
            impl ::core::default::Default for #enum_ident {
                fn default() -> Self {
                    Self::#def_ident(::core::default::Default::default())
                }
            }
        }
    } else {
        quote::quote! {}
    };

    // ──────────────────────────────────────────────────────────────
    // Generate final token stream
    // ──────────────────────────────────────────────────────────────
    let expanded = quote::quote! {
        impl #enum_ident {
            #(#helper_fns)*
        }

        #default_impl
    };

    ::proc_macro::TokenStream::from(expanded)
}
