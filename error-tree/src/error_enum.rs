crate::ix!();

#[derive(Clone,Debug,PartialEq)]
pub struct ErrorEnum {
    pub attrs:      Vec<Attribute>,
    pub visibility: syn::Visibility,
    pub ident:      Ident, // Enum name
    pub variants:   Vec<ErrorVariant>, // Variants
}

impl ToTokens for ErrorEnum {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ErrorEnum {
            attrs,
            visibility,
            ident,
            variants: _,
        } = &self;

        // Process attributes
        let mut other_attrs = Vec::new();
        let mut derives = Vec::new();
        let mut has_partial_eq = false;

        for attr in &self.attrs {
            if attr.path().is_ident("derive") {
                let paths: syn::punctuated::Punctuated<syn::Path, syn::token::Comma> =
                    attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                        .unwrap_or_default();
                for path in paths.iter() {
                    if path.is_ident("PartialEq") {
                        has_partial_eq = true;
                    } else {
                        derives.push(path.clone());
                    }
                }
            } else {
                other_attrs.push(attr.clone());
            }
        }

        // Ensure `Debug` is included
        if !derives.iter().any(|path| path.is_ident("Debug")) {
            derives.push(parse_quote!(Debug));
        }

        // Generate enum definition
        let variant_defs = self.variant_defs();
        tokens.extend(quote! {
            #(#other_attrs)*
            #[derive(#(#derives),*)]
            #visibility enum #ident {
                #(#variant_defs),*
            }
        });

        // Generate impl Display
        let display_impl = self.generate_display_impl();
        tokens.extend(display_impl);

        // Conditionally generate PartialEq implementation
        if has_partial_eq {
            if let Some(partial_eq_impl) = self.generate_partial_eq_impl() {
                tokens.extend(partial_eq_impl);
            }
        }
    }
}


impl ErrorEnum {

    fn has_derive_partial_eq(&self) -> bool {
        for attr in &self.attrs {
            if attr.path().is_ident("derive") {
                let mut found = false;
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("PartialEq") {
                        found = true;
                    }
                    Ok(())
                }).expect("Failed to parse nested meta");
                if found {
                    return true;
                }
            }
        }
        false
    }

    pub fn variant_defs(&self) -> Vec<TokenStream2> {
        self.variants.iter().map(|variant| {
            let attrs = variant.attrs();

            match variant {
                // Basic variants remain unchanged
                ErrorVariant::Basic { ident, .. } => quote! {
                    #(#attrs)*
                    #ident
                },
                // Wrapped variants generate tuple variants
                ErrorVariant::Wrapped { ident, ty, .. } => quote! {
                    #(#attrs)*
                    #ident(#ty)
                },
                // Struct variants remain unchanged
                ErrorVariant::Struct { ident, fields, .. } => {
                    let field_defs: Vec<_> = fields.iter().map(|field| {
                        let ErrorField { ident, ty } = field;
                        quote! { #ident: #ty }
                    }).collect();
                    quote! {
                        #(#attrs)*
                        #ident { #(#field_defs),* }
                    }
                },
            }
        }).collect()
    }

    pub fn find_variant_name_wrapping_type(&self, ty: &Type) -> Option<Ident> {

        self.variants.iter().find_map(|variant| {
            match variant {
                ErrorVariant::Wrapped { attrs: _, ident, ty: wrapped_ty, .. }
                    if **wrapped_ty == *ty => Some(ident.clone()),
                _ => None,
            }
        })
    }

    fn generate_display_impl(&self) -> TokenStream2 {
        let ident = &self.ident;

        let arms: Vec<TokenStream2> = self.variants.iter().map(|variant| {
            let variant_ident = variant.ident();
            let display_format = variant.display_format();

            match variant {
                // Basic variants
                ErrorVariant::Basic { .. } => {
                    if let Some(format_str) = display_format {
                        quote! {
                            #ident::#variant_ident => write!(f, #format_str),
                        }
                    } else {
                        quote! {
                            #ident::#variant_ident => write!(f, stringify!(#variant_ident)),
                        }
                    }
                },
                // Wrapped variants now match tuple variants
                ErrorVariant::Wrapped { .. } => {
                    if let Some(format_str) = display_format {
                        quote! {
                            #ident::#variant_ident(inner) => write!(f, #format_str, inner = inner),
                        }
                    } else {
                        quote! {
                            #ident::#variant_ident(inner) => write!(f, "{}: {:?}", stringify!(#variant_ident), inner),
                        }
                    }
                },
                // Struct variants
                ErrorVariant::Struct { fields, .. } => {
                    let field_idents: Vec<_> = fields.iter().map(|field| &field.ident).collect();
                    let pattern = quote! { #ident::#variant_ident { #(#field_idents),* } };

                    if let Some(format_str) = display_format {
                        let format_args = field_idents.iter().map(|ident| quote! { #ident = #ident });
                        quote! {
                            #pattern => write!(f, #format_str, #(#format_args),*),
                        }
                    } else {
                        quote! {
                            #pattern => write!(f, stringify!(#variant_ident)),
                        }
                    }
                },
            }
        }).collect();

        quote! {
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#arms)*
                    }
                }
            }
        }
    }

    fn generate_partial_eq_impl(&self) -> Option<TokenStream2> {
        let ident = &self.ident;

        // Generate match arms for each variant
        let arms: Vec<TokenStream2> = self.variants.iter().map(|variant| {
            let variant_ident = variant.ident();
            let cmp_neq = variant.cmp_neq();

            match variant {
                ErrorVariant::Basic { .. } => {
                    if cmp_neq {
                        quote! {
                            (#ident::#variant_ident, #ident::#variant_ident) => false,
                        }
                    } else {
                        quote! {
                            (#ident::#variant_ident, #ident::#variant_ident) => true,
                        }
                    }
                },
                ErrorVariant::Wrapped { .. } => {
                    if cmp_neq {
                        quote! {
                            (#ident::#variant_ident(_), #ident::#variant_ident(_)) => false,
                        }
                    } else {
                        quote! {
                            (#ident::#variant_ident(a), #ident::#variant_ident(b)) => a == b,
                        }
                    }
                },
                ErrorVariant::Struct { fields, .. } => {
                    if cmp_neq {
                        quote! {
                            (#ident::#variant_ident { .. }, #ident::#variant_ident { .. }) => false,
                        }
                    } else {
                        // Compare each field
                        let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
                        let a_fields: Vec<_> = field_idents.iter()
                            .map(|ident| format_ident!("a_{}", ident))
                            .collect();
                        let b_fields: Vec<_> = field_idents.iter()
                            .map(|ident| format_ident!("b_{}", ident))
                            .collect();

                        let pattern_a = quote! { #ident::#variant_ident { #(#field_idents: #a_fields),* } };
                        let pattern_b = quote! { #ident::#variant_ident { #(#field_idents: #b_fields),* } };

                        let comparisons = a_fields.iter().zip(b_fields.iter())
                            .map(|(a, b)| quote! { #a == #b });

                        quote! {
                            (#pattern_a, #pattern_b) => {
                                #(#comparisons)&&*
                            },
                        }
                    }
                },
            }
        }).collect();

        // Fallback arm for variants that don't match
        let fallback_arm = quote! {
            _ => false,
        };

        Some(quote! {
            impl PartialEq for #ident {
                fn eq(&self, other: &Self) -> bool {
                    match (self, other) {
                        #(#arms)*
                        #fallback_arm
                    }
                }
            }
        })
    }
}

impl Parse for ErrorEnum {

    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse attributes
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        // Parse visibility specifier (like `pub`)
        let visibility: syn::Visibility = input.parse()?;

        // Parse the `enum` keyword
        let _enum_token: Token![enum] = input.parse()?;

        // Parse the identifier (name of the enum)
        let ident: Ident = input.parse()?;

        // Parse the curly braces and the content within them
        let content;
        let _ = braced!(content in input);

        let mut variants: Vec<ErrorVariant> = Vec::new();
        while !content.is_empty() {
            let variant = content.parse()?;
            variants.push(variant);
            // Check for a trailing comma
            let _ = content.parse::<Option<Token![,]>>();
        }

        Ok(ErrorEnum {
            attrs,
            visibility,
            ident,
            variants,
        })
    }
}

impl Validate for ErrorEnum {
    fn validate(&self) -> bool {
        for variant in &self.variants {
            match variant {
                ErrorVariant::Basic { .. } => {},
                ErrorVariant::Wrapped { ty, .. } => {
                    if !ty.validate() {
                        return false;
                    }
                },
                ErrorVariant::Struct { fields, .. } => {
                    for field in fields {
                        if !field.ty.validate() {
                            return false;
                        }
                    }
                },
            }
        }
        true
    }
}

#[cfg(test)]
mod test_error_enum {

    use super::*;
    use syn::{parse_str, Ident, parse_quote};
    use proc_macro2::Span;

    #[test]
    fn test_parse() {
        let input_str = r#"
            #[derive(Clone)]
            pub enum FirstError {
                FormatError,
                IOError(std::io::Error),
                DeviceNotAvailable { device_name: String }
            }
            #[derive(PartialEq)]
            pub enum SecondError {
                AnotherError
            }
        "#;

        let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

        match parse_result {
            Ok(parsed_tree) => println!("Parsed successfully: {:#?}", parsed_tree),
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }

    fn test_error_enum(input_str: &str, vis: syn::Visibility, ident: Ident, variants: Vec<ErrorVariant>) {
        match parse_str::<ErrorEnum>(input_str) {
            Ok(parsed_enum) => {
                assert_eq!(parsed_enum.visibility, vis);
                assert_eq!(parsed_enum.ident, ident);
                assert_eq!(parsed_enum.variants, variants);
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }

    #[test]
    fn test_error_enum_parse_pub() {
        let input_str = "pub enum MyErrorEnum {
            FormatError,
            IOError(std::io::Error),
            DeviceNotAvailable { device_name: String }
        }";

        test_error_enum(input_str, parse_quote!(pub), Ident::new("MyErrorEnum", Span::call_site()), common_variants());
    }

    #[test]
    fn test_error_enum_parse_pub_super() {
        let input_str = "pub(super) enum MyErrorEnum {
            FormatError,
            IOError(std::io::Error),
            DeviceNotAvailable { device_name: String }
        }";

        test_error_enum(input_str, parse_quote!(pub(super)), Ident::new("MyErrorEnum", Span::call_site()), common_variants());
    }

    #[test]
    fn test_error_enum_parse_pub_crate() {
        let input_str = "pub(crate) enum MyErrorEnum {
            FormatError,
            IOError(std::io::Error),
            DeviceNotAvailable { device_name: String }
        }";

        test_error_enum(input_str, parse_quote!(pub(crate)), Ident::new("MyErrorEnum", Span::call_site()), common_variants());
    }

    #[test]
    fn test_error_enum_parse_no_vis() {
        let input_str = "enum MyErrorEnum {
            FormatError,
            IOError(std::io::Error),
            DeviceNotAvailable { device_name: String }
        }";

        test_error_enum(input_str, parse_quote!(), Ident::new("MyErrorEnum", Span::call_site()), common_variants());
    }

    fn common_variants() -> Vec<ErrorVariant> {
        vec![
            ErrorVariant::Basic{
                attrs: vec![],
                ident: Ident::new("FormatError", Span::call_site()),
                cmp_neq: false,
                display_format: None,
            },
            ErrorVariant::Wrapped{
                attrs: vec![],
                ident: Ident::new("IOError", Span::call_site()), 
                ty:    Box::new(parse_quote!(std::io::Error)),
                cmp_neq: false,
                display_format: None,
            },
            ErrorVariant::Struct{
                attrs:  vec![],
                ident:  Ident::new("DeviceNotAvailable", Span::call_site()), 
                fields: vec![
                    ErrorField {
                        ident: Ident::new("device_name", Span::call_site()),
                        ty: parse_quote!(String)
                    }
                ],
                cmp_neq: false,
                display_format: None,
            }
        ]
    }
}
