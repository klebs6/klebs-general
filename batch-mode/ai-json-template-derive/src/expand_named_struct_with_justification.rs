// ---------------- [ File: ai-json-template-derive/src/expand_named_struct_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_named_struct_with_justification(
    ty_ident: &syn::Ident,
    ds: &syn::DataStruct,
    span: proc_macro2::Span,
    container_docs_str: &str,
) -> proc_macro2::TokenStream {
    trace!("Handling named struct justification expansions for '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();
    match &ds.fields {
        syn::Fields::Named(named_fields) => {
            let (just_ts, conf_ts, justified_ts, accessor_ts) =
                generate_justified_structs_for_named(ty_ident, named_fields, span);

            let tpls = generate_to_template_with_justification_for_named(
                ty_ident,
                named_fields,
                container_docs_str
            );

            out.extend(just_ts);
            out.extend(conf_ts);
            out.extend(justified_ts);
            out.extend(accessor_ts);
            out.extend(tpls);
        }
        _ => {
            warn!("Struct is not named => returning error");
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification only supports named structs"
            );
            out.extend(e.to_compile_error());
        }
    }
    out
}

#[cfg(test)]
mod check_expand_named_struct_with_justification {
    use super::*;

    #[traced_test]
    fn verify_error_on_non_named_struct() {
        trace!("Constructing a tuple-struct-like DataStruct to provoke an error path.");

        // We'll parse an entire item struct that is tuple-like:
        //    struct MyTuple(i32, String);
        //
        // Then from that, we'll build a `DataStruct` by pulling out the fields.
        let item_struct: ItemStruct = parse_quote! {
            struct MyTuple(i32, String);
        };

        // Build the DataStruct manually using the fields from `item_struct`.
        let ds = DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields, // these are the Unnamed fields
            semi_token: item_struct.semi_token,
        };

        let container_docs_str = "Simulating a tuple struct for error scenario.";

        trace!("Invoking expand_named_struct_with_justification with a tuple-struct data.");
        let token_stream = expand_named_struct_with_justification(
            &item_struct.ident,
            &ds,
            item_struct.ident.span(),
            container_docs_str
        );

        debug!("Resulting expansion: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("compile_error") 
             || ts_str.contains("AiJsonTemplateWithJustification only supports named structs"),
            "Expected an error expansion for non-named struct, but got: {ts_str}"
        );
        info!("verify_error_on_non_named_struct passed.");
    }

    #[traced_test]
    fn verify_named_struct_no_fields() {
        trace!("Constructing an empty named struct to validate expansions.");

        // This simulates:
        //    struct MyEmptyStruct {}
        //
        // We'll parse an entire ItemStruct and then build DataStruct from it.
        let item_struct: ItemStruct = parse_quote! {
            struct MyEmptyStruct {}
        };

        let ds = DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields,
            semi_token: item_struct.semi_token,
        };

        let container_docs_str = "No fields struct test.";

        trace!("Invoking expand_named_struct_with_justification for an empty named struct.");
        let token_stream = expand_named_struct_with_justification(
            &item_struct.ident,
            &ds,
            item_struct.ident.span(),
            container_docs_str
        );

        debug!("Resulting expansion: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("JustifiedMyEmptyStruct"),
            "Expected the Justified type name to appear for an empty struct. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MyEmptyStructJustification"),
            "Expected justification struct name to appear. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MyEmptyStructConfidence"),
            "Expected confidence struct name to appear. Got: {ts_str}"
        );
        info!("verify_named_struct_no_fields passed.");
    }

    #[traced_test]
    fn verify_named_struct_single_field() {
        trace!("Constructing a named struct with one field to validate expansions.");

        // We'll parse a full item struct, e.g.:
        //    struct MySingleField {
        //        alpha: i32
        //    }
        let item_struct: ItemStruct = parse_quote! {
            struct MySingleField {
                alpha: i32
            }
        };

        let ds = DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields,
            semi_token: item_struct.semi_token,
        };

        let container_docs_str = "Single field struct test.";

        trace!("Invoking expand_named_struct_with_justification for a single-field named struct.");
        let token_stream = expand_named_struct_with_justification(
            &item_struct.ident,
            &ds,
            item_struct.ident.span(),
            container_docs_str
        );

        debug!("Resulting expansion: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("JustifiedMySingleField"),
            "Expected the Justified type name in the expansion. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MySingleFieldJustification"),
            "Expected justification struct name to appear. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MySingleFieldConfidence"),
            "Expected confidence struct name to appear. Got: {ts_str}"
        );

        // Since we have one field "alpha", we should see a justification/confidence reference for it:
        assert!(
            ts_str.contains("alpha_justification") && ts_str.contains("alpha_confidence"),
            "Expected alpha_justification/alpha_confidence fields. Got: {ts_str}"
        );
        info!("verify_named_struct_single_field passed.");
    }

    #[traced_test]
    fn verify_named_struct_multiple_fields_justified() {
        trace!("Constructing a named struct with multiple fields (some with justification turned off).");

        // We'll parse a full item struct. 
        // e.g.:
        //    struct MixedStruct {
        //        #[justify = false]
        //        beta: bool,
        //        gamma: String
        //    }
        let item_struct: ItemStruct = parse_quote! {
            struct MixedStruct {
                #[justify = false]
                beta: bool,
                gamma: String
            }
        };

        let ds = DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields,
            semi_token: item_struct.semi_token,
        };

        let container_docs_str = "Multiple fields struct test, partial justification";

        trace!("Invoking expand_named_struct_with_justification for multiple fields, with partial justification disabled.");
        let token_stream = expand_named_struct_with_justification(
            &item_struct.ident,
            &ds,
            item_struct.ident.span(),
            container_docs_str
        );

        debug!("Resulting expansion: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        // We always create these top-level types:
        assert!(
            ts_str.contains("JustifiedMixedStruct"),
            "Expected the Justified type name in the expansion. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MixedStructJustification"),
            "Expected justification struct name to appear. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("MixedStructConfidence"),
            "Expected confidence struct name to appear. Got: {ts_str}"
        );

        // For field 'beta' => justification is disabled
        // For field 'gamma' => justification is enabled
        assert!(
            !ts_str.contains("beta_justification") && !ts_str.contains("beta_confidence"),
            "Field 'beta' has #[justify=false], so should not have just/conf. Got: {ts_str}"
        );
        assert!(
            ts_str.contains("gamma_justification") && ts_str.contains("gamma_confidence"),
            "Field 'gamma' should have just/conf expansions. Got: {ts_str}"
        );
        info!("verify_named_struct_multiple_fields_justified passed.");
    }

    #[traced_test]
    fn verify_named_struct_with_doc_comments() {
        trace!("Constructing a named struct that includes doc comments to ensure they're captured in container_docs_str.");

        // We'll parse a full item struct:
        //    /// This is MyDocStruct
        //    /// second line
        //    struct MyDocStruct {
        //        x: i64
        //    }
        //
        // In real usage, gather_doc_comments would see them. We'll simulate it by passing
        // "This is MyDocStruct\nsecond line" as container_docs_str.

        let item_struct: ItemStruct = parse_quote! {
            struct MyDocStruct {
                x: i64
            }
        };

        let ds = DataStruct {
            struct_token: item_struct.struct_token,
            fields: item_struct.fields,
            semi_token: item_struct.semi_token,
        };

        let container_docs_str = "This is MyDocStruct\nsecond line";

        trace!("Invoking expand_named_struct_with_justification for a doc-laden struct");
        let token_stream = expand_named_struct_with_justification(
            &item_struct.ident,
            &ds,
            item_struct.ident.span(),
            container_docs_str
        );

        debug!("Resulting expansion: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("JustifiedMyDocStruct"),
            "Expected 'JustifiedMyDocStruct' in expansions. Got: {ts_str}"
        );
        // We won't do a deep test on doc string usage, but let's confirm we see the doc lines:
        assert!(
            ts_str.contains("This is MyDocStruct") && ts_str.contains("second line"),
            "Expected container doc lines to appear somewhere in expansions. Got: {ts_str}"
        );
        info!("verify_named_struct_with_doc_comments passed.");
    }
}
