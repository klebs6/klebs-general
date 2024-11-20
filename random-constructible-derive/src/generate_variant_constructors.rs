crate::ix!();

/// Generates constructors for each variant of an enum.
///
/// # Arguments
///
/// * `enum_name` - The identifier of the enum.
/// * `variant_idents` - A vector of variant identifiers.
/// * `variant_fields` - A vector of fields corresponding to each variant.
///
/// # Returns
///
/// A vector of `TokenStream2` representing the constructors for each variant.
pub fn generate_variant_constructors(
    enum_name: &Ident,
    variant_idents: &[Ident],
    variant_fields: &[Fields],
) -> Vec<TokenStream2> {
    variant_idents
        .iter()
        .zip(variant_fields.iter())
        .map(|(ident, fields)| match fields {
            Fields::Unit => {
                quote! { #enum_name::#ident }
            }
            Fields::Unnamed(fields_unnamed) => {
                let field_types = fields_unnamed.unnamed.iter().map(|f| &f.ty);
                let field_values = field_types.map(|ty| quote! { <#ty as RandConstruct>::random() });
                quote! {
                    #enum_name::#ident( #(#field_values),* )
                }
            }
            Fields::Named(fields_named) => {
                let field_names = fields_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap());
                let field_types = fields_named.named.iter().map(|f| &f.ty);
                let field_values = field_types.map(|ty| quote! { <#ty as RandConstruct>::random() });
                quote! {
                    #enum_name::#ident {
                        #(#field_names: #field_values),*
                    }
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Fields, FieldsNamed, FieldsUnnamed};

    #[test]
    fn test_generate_variant_constructors() {
        // Define the enum name
        let enum_name: Ident = parse_quote! { MyEnum };

        // Define variant identifiers
        let variant_idents: Vec<Ident> = vec![
            parse_quote! { UnitVariant },
            parse_quote! { UnnamedVariant },
            parse_quote! { NamedVariant },
        ];

        // Define variant fields
        let variant_fields: Vec<Fields> = vec![
            // Unit variant
            Fields::Unit,
            // Unnamed variant with two fields
            Fields::Unnamed(parse_quote! {
                (i32, String)
            }),
            // Named variant with two fields
            Fields::Named(parse_quote! {
                {
                    x: f64,
                    y: bool
                }
            }),
        ];

        // Generate the constructors
        let constructors = generate_variant_constructors(&enum_name, &variant_idents, &variant_fields);

        // Convert the TokenStreams to strings for assertion
        let constructor_strings: Vec<String> = constructors.iter().map(|ts| ts.to_string()).collect();

        // Expected constructors
        let expected_constructors = vec![
            "MyEnum :: UnitVariant",
            "MyEnum :: UnnamedVariant (< i32 as RandConstruct > :: random () , < String as RandConstruct > :: random ())",
            "MyEnum :: NamedVariant { x : < f64 as RandConstruct > :: random () , y : < bool as RandConstruct > :: random () }",
        ];

        // Assert that the generated constructors match the expected ones
        assert_eq!(constructor_strings, expected_constructors);
    }
}
