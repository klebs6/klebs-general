crate::ix!();

/// Generate match arms for all variants in an enum.
pub fn generate_variant_arms(
    enum_name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> Vec<TokenStream2> {
    variants
        .iter()
        .map(|variant| generate_variant_arm(enum_name, variant))
        .collect()
}

#[test]
fn test_generate_variant_arms() {
    let enum_name: Ident = parse_quote!(MyEnum);
    let variants: Punctuated<Variant, Comma> = parse_quote! {
        NamedVariant { field1: i32, field2: String },
        UnnamedVariant(i32, String),
        UnitVariant
    };

    let tokens = generate_variant_arms(&enum_name, &variants);

    let expected: Vec<TokenStream2> = vec![
        quote! {
            MyEnum::NamedVariant { field1, field2 } => {
                let field1_ai = format!("{}", field1);
                let field2_ai = format!("{}", field2);
                let description = format!("NamedVariant {{ field1: {{}}, field2: {{}} }}", field1_ai, field2_ai);
                std::borrow::Cow::Owned(description)
            }
        },
        quote! {
            MyEnum::UnnamedVariant(field0, field1) => {
                let field0_ai = format!("{}", field0);
                let field1_ai = format!("{}", field1);
                let description = format!("UnnamedVariant({{}}, {{}})", field0_ai, field1_ai);
                std::borrow::Cow::Owned(description)
            }
        },
        quote! {
            MyEnum::UnitVariant => std::borrow::Cow::Borrowed(stringify!(UnitVariant))
        },
    ];

    assert_eq!(
        tokens.iter().map(|ts| ts.to_string()).collect::<Vec<_>>(),
        expected.iter().map(|ts| ts.to_string()).collect::<Vec<_>>()
    );
}
