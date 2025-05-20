crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_manual_default_for_justified_named_struct(
    base_ty_ident:  &syn::Ident,
    named_fields:   &syn::FieldsNamed
) -> proc_macro2::TokenStream
{
    let justified_ident = syn::Ident::new(
        &format!("Justified{}", base_ty_ident),
        base_ty_ident.span()
    );

    // Build a series of `field: Default::default()`, plus 
    // `field_confidence: 0.0, field_justification: String::new()`
    // plus top-level items if present (like additional fields for skip_self_just=false).
    let mut field_inits = Vec::new();

    for field in &named_fields.named {
        let field_ident = match &field.ident {
            Some(i) => i,
            None => {
                continue;
            }
        };

        let name_s = field_ident.to_string();
        if name_s.ends_with("_justification") {
            field_inits.push(quote::quote! {
                #field_ident: ::std::string::String::new()
            });
        } else if name_s.ends_with("_confidence") {
            field_inits.push(quote::quote! {
                #field_ident: 0.0
            });
        } else {
            // It's an actual subfield => .default()
            field_inits.push(quote::quote! {
                #field_ident: ::core::default::Default::default()
            });
        }
    }

    let expanded = quote::quote! {
        impl ::core::default::Default for #justified_ident {
            fn default() -> Self {
                Self {
                    #( #field_inits ),*
                }
            }
        }
    };
    expanded
}

#[cfg(test)]
mod test_generate_manual_default_for_justified_named_struct {
    use super::*;

    #[traced_test]
    fn test_basic_named_struct_default() {
        trace!("test_basic_named_struct_default: starting");
        let base_ident = syn::Ident::new("MyStruct", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote! {
            {
                foo: String,
                foo_confidence: f64,
                foo_justification: String,
                bar: i32,
                bar_confidence: f64,
                bar_justification: String
            }
        };
        let ts = generate_manual_default_for_justified_named_struct(&base_ident, &named_fields);
        debug!("Generated:\n{}", ts.to_string());
        let out_str = ts.to_string();
        assert!(
            out_str.contains("JustifiedMyStruct"),
            "Should match naming convention? Actually we re-check usage: we might want Justified + base name"
        );
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedMyStruct"),
            "Should implement Default on JustifiedMyStruct"
        );
        assert!(
            out_str.contains("foo : :: core :: default :: Default :: default ()"),
            "Should set child fields to .default()"
        );
        assert!(
            out_str.contains("foo_confidence : 0.0"),
            "Should set _confidence fields to 0.0"
        );
        assert!(
            out_str.contains("foo_justification : :: std :: string :: String :: new ()"),
            "Should set _justification fields to String::new()"
        );
    }

    #[traced_test]
    fn test_struct_some_fields_skip() {
        trace!("test_struct_some_fields_skip: starting");
        let base_ident = syn::Ident::new("S", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote! {
            {
                #[serde(default)]
                x: i32,
                x_confidence: f64,
                x_justification: String,
                #[justify(false)]
                y: bool
            }
        };
        // Although y may skip justification, it's still present in the flattened struct as 'y'.
        let ts = generate_manual_default_for_justified_named_struct(&base_ident, &named_fields);
        debug!("Generated: {}", ts.to_string());
        let code_str = ts.to_string();
        assert!(
            code_str.contains("impl :: core :: default :: Default for JustifiedS"),
            "Should implement default for JustifiedS"
        );
        // x => defaults
        assert!(code_str.contains("x : :: core :: default :: Default :: default ()"), "x => default");
        assert!(code_str.contains("x_confidence : 0.0"), "x_confidence => 0.0");
        assert!(code_str.contains("x_justification : :: std :: string :: String :: new ()"), "x_just => empty");
        // y => default
        assert!(code_str.contains("y : :: core :: default :: Default :: default ()"), "y => default, even if justify=false");
    }

    #[traced_test]
    fn test_sanity_parse_code() {
        trace!("test_sanity_parse_code: starting");
        let base_ident = syn::Ident::new("TestStruct", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote! {
            {
                a: u8,
                a_confidence: f64,
                a_justification: String
            }
        };
        let ts = generate_manual_default_for_justified_named_struct(&base_ident, &named_fields);
        let code_str = ts.to_string();
        let parse_res: syn::Result<syn::File> = syn::parse_str(&code_str);
        assert!(parse_res.is_ok(), "Generated code must parse as valid Rust");
    }

    #[traced_test]
    fn test_zero_fields() {
        trace!("test_zero_fields: starting");
        let base_ident = syn::Ident::new("Empty", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote! { {} };
        let ts = generate_manual_default_for_justified_named_struct(&base_ident, &named_fields);
        let out_str = ts.to_string();
        debug!("Output: {}", out_str);
        // Should just produce an empty default impl with no fields
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedEmpty"),
            "Should impl Default for an empty struct"
        );
        assert!(
            out_str.contains("Self { }"),
            "Should have no fields"
        );
    }
}
