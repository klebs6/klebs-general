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
        let ErrorEnum { attrs, visibility, ident, variants: _ } = &self;

        // Generate enum definitions
        let variant_defs = self.variant_defs();

        tokens.extend(
            quote! {
                // Enum definition
                #(#attrs)*
                #[derive(Debug)]
                #visibility enum #ident {
                    #(#variant_defs),*
                }
            }
        )
    }
}

impl ErrorEnum {

    pub fn find_variant_name_wrapping_type(&self, ty: &Type) -> Option<Ident> {

        self.variants.iter().find_map(|variant| {
            match variant {
                ErrorVariant::Wrapped(ident, wrapped_ty) 
                    if **wrapped_ty == *ty => Some(ident.clone()),
                _ => None,
            }
        })
    }

    pub fn variant_defs(&self) -> Vec<TokenStream2> {
        self.variants.iter().map(|variant| {
            match variant {
                ErrorVariant::Basic(ident) => quote! { #ident },
                ErrorVariant::Wrapped(ident, ty) => quote! { #ident(#ty) },
                ErrorVariant::Struct(ident, fields) => {
                    let field_defs: Vec<_> = fields.iter().map(|field| {
                        let ErrorField { ident, ty } = field;
                        quote! { #ident: #ty }
                    }).collect();
                    quote! { #ident { #(#field_defs),* } }
                }
            }
        }).collect()
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

        // Check variant validity
        for variant in &self.variants {

            match variant {

                ErrorVariant::Basic(_) => {}, // Basic variants are usually always valid

                ErrorVariant::Wrapped(_, ty) => {
                    if !ty.validate() {
                        return false;
                    }
                },

                ErrorVariant::Struct(_, fields) => {

                    for field in fields {

                        if !field.ty.validate() {
                            return false;
                        }
                    }
                },
            }
        }

        // Additional checks can be implemented here
        // ...

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
            ErrorVariant::Basic(Ident::new("FormatError", Span::call_site())),
            ErrorVariant::Wrapped(Ident::new("IOError", Span::call_site()), Box::new(parse_quote!(std::io::Error))),
            ErrorVariant::Struct(Ident::new("DeviceNotAvailable", Span::call_site()), vec![
                ErrorField {
                    ident: Ident::new("device_name", Span::call_site()),
                    ty: parse_quote!(String)
                }
            ])
        ]
    }
}
