crate::ix!();

#[derive(Clone,Debug,PartialEq)]
pub struct ErrorEnum {
    pub visibility: syn::Visibility,
    pub ident:      Ident, // Enum name
    pub variants:   Vec<ErrorVariant>, // Variants
}

impl ToTokens for ErrorEnum {

    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let ErrorEnum { visibility, ident, variants: _ } = &self;

        // Generate enum definitions
        let variant_defs = self.variant_defs();

        tokens.extend(
            quote! {
                // Enum definition
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

        // Parsing visibility specifier (like `pub`)
        let visibility: syn::Visibility = input.parse()?;

        // Parsing the `enum` keyword
        let _enum_token: Token![enum] = input.parse()?;

        // Parsing the identifier (name of the enum)
        let ident: Ident = input.parse()?;

        // Parsing the curly braces and the content within them
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
    use syn::{parse_str, Ident, Type, parse_quote};
    use proc_macro2::Span;

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
