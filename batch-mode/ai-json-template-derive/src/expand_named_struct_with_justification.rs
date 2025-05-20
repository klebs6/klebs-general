// ---------------- [ File: ai-json-template-derive/src/expand_named_struct_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_named_struct_with_justification(
    ty_ident:           &syn::Ident,
    ds:                 &syn::DataStruct,
    span:               proc_macro2::Span,
    container_docs_str: &str
) -> proc_macro2::TokenStream
{
    trace!("expand_named_struct_with_justification => '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    match &ds.fields {
        syn::Fields::Named(named_fields) => {
            // 1) Build the single flattened “JustifiedFoo” struct
            let flattened_ts = generate_justified_structs_for_named(ty_ident, named_fields, span);
            out.extend(flattened_ts);

            // 2) Generate the `impl AiJsonTemplateWithJustification` that
            //    includes `to_template_with_justification()`
            let to_tpl_ts = generate_to_template_with_justification_for_named(
                ty_ident,
                named_fields,
                container_docs_str
            );
            out.extend(to_tpl_ts);

            // 3) Generate the reverse `impl From<JustifiedFoo> for Foo`
            let from_impl = generate_reverse_from_impl_for_named_with_justification(
                ty_ident,
                named_fields,
                span,
            );
            out.extend(from_impl);

            // 4) Generate an `impl Default for JustifiedFoo` that sets 
            //    - original subfields => .default()
            //    - any "X_confidence" => 0.0
            //    - any "X_justification" => String::new()
            let default_impl_ts = generate_manual_default_for_justified_named_struct(ty_ident, named_fields);
            out.extend(default_impl_ts);
        }
        _ => {
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification only supports named structs"
            );
            out.extend(e.to_compile_error());
        }
    }

    trace!("expand_named_struct_with_justification => done '{}'", ty_ident);
    out
}

#[cfg(test)]
mod test_expand_named_struct_with_justification {
    use super::*;

    #[traced_test]
    fn test_basic_named_struct() {
        trace!("test_basic_named_struct: starting");
        let input: DeriveInput = parse_quote! {
            /// doc for MyStruct
            struct MyStruct {
                foo: i32,
                bar: String,
            }
        };
        let container_docs_str = "Container docs for MyStruct";
        let data_struct = match &input.data {
            syn::Data::Struct(ds) => ds,
            _ => panic!("Expected a struct"),
        };
        let output = expand_named_struct_with_justification(&input.ident, data_struct, input.span(), container_docs_str);
        debug!("Generated:\n{}", output.to_string());
        let out_str = output.to_string();
        assert!(
            out_str.contains("pub struct JustifiedMyStruct"),
            "Should create flattened struct JustifiedMyStruct"
        );
        assert!(
            out_str.contains("impl AiJsonTemplateWithJustification for MyStruct"),
            "Should implement AiJsonTemplateWithJustification"
        );
        assert!(
            out_str.contains("impl :: core :: convert :: From < JustifiedMyStruct > for MyStruct"),
            "Should implement From<JustifiedMyStruct> for MyStruct"
        );
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedMyStruct"),
            "Should implement Default for JustifiedMyStruct"
        );
    }

    #[traced_test]
    fn test_struct_fields_justify_false() {
        trace!("test_struct_fields_justify_false: starting");
        let input: DeriveInput = parse_quote! {
            struct S {
                #[justify(false)]
                x: i32,
                y: String
            }
        };
        let data_struct = match &input.data {
            syn::Data::Struct(ds) => ds,
            _ => panic!("Expected a struct"),
        };
        let output = expand_named_struct_with_justification(&input.ident, data_struct, input.span(), "Docs for S");
        let out_str = output.to_string();
        assert!(
            out_str.contains("pub struct JustifiedS { x :"),
            "Should still flatten 'x'"
        );
        assert!(
            !out_str.contains("x_confidence"),
            "Should skip x_confidence"
        );
        assert!(
            !out_str.contains("x_justification"),
            "Should skip x_justification"
        );
        assert!(
            out_str.contains("y_confidence"),
            "Should have y_confidence"
        );
        assert!(
            out_str.contains("y_justification"),
            "Should have y_justification"
        );
    }

    #[traced_test]
    fn test_struct_with_option_vec_hashmap() {
        trace!("test_struct_with_option_vec_hashmap: starting");
        let input: DeriveInput = parse_quote! {
            struct Complex {
                nums: Vec<u8>,
                maybe: Option<String>,
                map: std::collections::HashMap<String, bool>,
            }
        };
        let data_struct = match &input.data {
            syn::Data::Struct(ds) => ds,
            _ => panic!("Expected a struct"),
        };
        let ts = expand_named_struct_with_justification(&input.ident, data_struct, input.span(), "Docs for Complex");
        let out_str = ts.to_string();
        debug!("Generated: {}", out_str);
        // Check that each field is flattened with conf/just
        assert!(out_str.contains("nums : Vec < u8"), "Should produce standard Vec<u8>");
        assert!(out_str.contains("maybe : Option < String"), "Should produce standard Option<String>");
        assert!(out_str.contains("map : std :: collections :: HashMap < String , bool"),
            "Should produce standard type for HashMap<String,bool>");
    }

    #[traced_test]
    fn test_sanity_parse_generated_code() {
        trace!("test_sanity_parse_generated_code: starting");
        let input: DeriveInput = parse_quote! {
            struct SomeStruct {
                alpha: bool,
                beta: String
            }
        };
        let ds = match &input.data {
            syn::Data::Struct(ds) => ds,
            _ => panic!("Expected a struct"),
        };
        let code_ts = expand_named_struct_with_justification(&input.ident, ds, input.span(), "Docs for SomeStruct");
        let code_str = code_ts.to_string();
        let parse_res: syn::Result<syn::File> = syn::parse_str(&code_str);
        assert!(
            parse_res.is_ok(),
            "Generated code must parse as valid Rust"
        );
    }
}
