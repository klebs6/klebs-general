crate::ix!();

// Function to check if a type is a primitive type
pub fn is_primitive_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(segment) = path.segments.last() {
                let ident = &segment.ident;
                match ident.to_string().as_str() {
                    // Numeric types
                    "u8" | "u16" | "u32" | "u64" | "u128" |
                    "i8" | "i16" | "i32" | "i64" | "i128" |
                    "f32" | "f64" |
                    // Other primitives
                    "bool" | "char" => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        // Handle unit type
        Type::Tuple(tuple) => tuple.elems.is_empty(),
        _ => false,
    }
}

// Function to check if any field is a primitive type
pub fn contains_primitive_type(field_types: &[Type]) -> bool {
    field_types.iter().any(|ty| is_primitive_type(ty))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_primitive_type() {
        // Numeric types
        assert!(is_primitive_type(&parse_quote! { u8 }));
        assert!(is_primitive_type(&parse_quote! { u16 }));
        assert!(is_primitive_type(&parse_quote! { u32 }));
        assert!(is_primitive_type(&parse_quote! { u64 }));
        assert!(is_primitive_type(&parse_quote! { u128 }));

        assert!(is_primitive_type(&parse_quote! { i8 }));
        assert!(is_primitive_type(&parse_quote! { i16 }));
        assert!(is_primitive_type(&parse_quote! { i32 }));
        assert!(is_primitive_type(&parse_quote! { i64 }));
        assert!(is_primitive_type(&parse_quote! { i128 }));

        assert!(is_primitive_type(&parse_quote! { f32 }));
        assert!(is_primitive_type(&parse_quote! { f64 }));

        // Other primitives
        assert!(is_primitive_type(&parse_quote! { bool }));
        assert!(is_primitive_type(&parse_quote! { char }));

        // Special unit type
        assert!(is_primitive_type(&parse_quote! { () }));

        // Non-primitive types
        assert!(!is_primitive_type(&parse_quote! { String }));
        assert!(!is_primitive_type(&parse_quote! { Vec<u8> }));
        assert!(!is_primitive_type(&parse_quote! { Option<i32> }));
    }

    #[test]
    fn test_contains_primitive_type() {
        let field_types = vec![
            parse_quote! { String },
            parse_quote! { Vec<u8> },
            parse_quote! { u8 },
        ];
        assert!(contains_primitive_type(&field_types));

        let field_types = vec![
            parse_quote! { String },
            parse_quote! { Vec<u8> },
        ];
        assert!(!contains_primitive_type(&field_types));
    }

    #[test]
    fn test_with_struct_fields() {
        let fields: syn::FieldsNamed = parse_quote! {
            {
                field1: u8,
                field2: String,
                field3: f32,
            }
        };

        let field_types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
        assert!(contains_primitive_type(&field_types));

        let fields: syn::FieldsNamed = parse_quote! {
            {
                field1: String,
                field2: Vec<u8>,
            }
        };

        let field_types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
        assert!(!contains_primitive_type(&field_types));
    }
}
