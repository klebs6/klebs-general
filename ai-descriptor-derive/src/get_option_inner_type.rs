crate::ix!();

pub(crate) fn get_option_inner_type(ty: &Type) -> &Type {
    if let Type::Path(TypePath { path, .. }) = ty {
        for segment in &path.segments {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty;
                    }
                }
            }
        }
    }
    ty
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Type};

    #[test]
    fn test_get_option_inner_type_with_option() {
        let ty: Type = parse_quote!(Option<String>);
        let inner_ty = get_option_inner_type(&ty);
        let expected_ty: Type = parse_quote!(String);
        assert_eq!(inner_ty.to_token_stream().to_string(), expected_ty.to_token_stream().to_string());
    }

    #[test]
    fn test_get_option_inner_type_with_non_option() {
        let ty: Type = parse_quote!(String);
        let inner_ty = get_option_inner_type(&ty);
        assert_eq!(inner_ty.to_token_stream().to_string(), ty.to_token_stream().to_string());
    }

    #[test]
    fn test_get_option_inner_type_with_nested_option() {
        let ty: Type = parse_quote!(Option<Option<String>>);
        let inner_ty = get_option_inner_type(&ty);
        let expected_ty: Type = parse_quote!(Option<String>);
        assert_eq!(inner_ty.to_token_stream().to_string(), expected_ty.to_token_stream().to_string());
    }

    #[test]
    fn test_get_option_inner_type_with_result() {
        let ty: Type = parse_quote!(Result<String, Error>);
        let inner_ty = get_option_inner_type(&ty);
        assert_eq!(inner_ty.to_token_stream().to_string(), ty.to_token_stream().to_string());
    }

    #[test]
    fn test_get_option_inner_type_with_reference_option() {
        let ty: Type = parse_quote!(&Option<String>);
        let inner_ty = get_option_inner_type(&ty);
        assert_eq!(inner_ty.to_token_stream().to_string(), ty.to_token_stream().to_string());
    }

    #[test]
    fn test_get_option_inner_type_with_std_option() {
        let ty: Type = parse_quote!(std::option::Option<String>);
        let inner_ty = get_option_inner_type(&ty);
        let expected_ty: Type = parse_quote!(String);
        assert_eq!(inner_ty.to_token_stream().to_string(), expected_ty.to_token_stream().to_string());
    }
}

