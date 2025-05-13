// ---------------- [ File: ai-json-template-derive/src/gather_schemas_and_placeholders_for_named_fields.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn gather_schemas_and_placeholders_for_named_fields(
    fields: &syn::FieldsNamed
) -> Vec<proc_macro2::TokenStream> {
    use tracing::debug;

    let mut expansions = Vec::new();
    for field in &fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                debug!("Skipping unnamed field in a named struct?");
                continue;
            }
        };
        let field_name_str = field_ident.to_string();
        debug!("Processing field '{}'", field_name_str);

        let doc_str = gather_doc_comments(&field.attrs).join("\n");
        let is_required = extract_option_inner(&field.ty).is_none();
        let skip_self_just   = is_justification_disabled_for_field(field);
        let skip_child_just  = skip_self_just || is_justification_disabled_for_inner(field);

        // (A) Normal child schema
        if let Some(child_expr) = build_named_field_child_schema_expr(field, &doc_str, is_required, skip_child_just) {
            expansions.push(quote::quote! {
                map.insert(#field_name_str.to_string(), #child_expr);
            });
        }

        // (B) Just/conf placeholders if skip_self_just is false
        if !skip_self_just {
            let just_conf_ts = build_named_field_just_conf_placeholders(&field_name_str);
            expansions.push(just_conf_ts);
        }
    }
    expansions
}

#[cfg(test)]
mod tests_for_gather_schemas_and_placeholders_for_named_fields {
    use super::*;

    /// Helper function to parse a string representing a struct
    /// and extract its `FieldsNamed` for testing.
    /// We avoid `pub` in the struct per requirements.
    fn parse_named_fields(struct_src: &str) -> FieldsNamed {
        trace!("Parsing struct source:\n{}", struct_src);
        let parsed: ItemStruct = syn::parse_str(struct_src).expect("Unable to parse struct");
        match &parsed.fields {
            Fields::Named(named) => {
                debug!("Struct parse successful => returning named fields");
                named.clone()
            }
            _ => {
                error!("Parsed struct but did not have named fields");
                panic!("Expected named fields, got something else");
            }
        }
    }

    /// Helper to display the returned `TokenStream`s for debugging.
    /// (We only do a partial textual check in the tests, but this is
    /// helpful for logging the entire expansions.)
    fn tokens_to_string_vec(ts_vec: &[proc_macro2::TokenStream]) -> Vec<String> {
        ts_vec.iter().map(|ts| ts.to_string()).collect()
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_empty() {
        trace!("Starting test_gather_schemas_and_placeholders_empty");
        let src = r#"
            struct EmptyTestStruct {}
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        debug!("Result expansions count: {}", expansions.len());
        assert!(expansions.is_empty(), "Expected no expansions for empty fields");
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_single_field_no_doc() {
        trace!("Starting test_gather_schemas_and_placeholders_single_field_no_doc");
        let src = r#"
            struct SingleFieldNoDoc {
                foo: String
            }
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        let expansions_str = tokens_to_string_vec(&expansions);
        debug!("Expansions: {:#?}", expansions_str);

        // We expect two expansions:
        //  1) The child's normal schema insertion
        //  2) The "just/conf" placeholders for "foo"
        assert_eq!(expansions.len(), 2, "Should have exactly two expansions");

        // Check that the normal schema mentions 'foo'
        assert!(
            expansions_str[0].contains("map.insert(\"foo\""),
            "First expansion should insert 'foo' child schema"
        );

        // Check that the second references 'foo_justification' or 'foo_confidence'
        let combined = expansions_str.join("\n");
        assert!(
            combined.contains("foo_justification") && combined.contains("foo_confidence"),
            "Should contain placeholders for 'foo_justification' and 'foo_confidence'"
        );
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_single_field_with_doc() {
        trace!("Starting test_gather_schemas_and_placeholders_single_field_with_doc");
        let src = r#"
            struct SingleFieldWithDoc {
                /// A doc line for bar
                bar: Option<i32>
            }
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        let expansions_str = tokens_to_string_vec(&expansions);
        debug!("Expansions: {:#?}", expansions_str);

        // We expect two expansions again:
        //  1) The child's normal schema insertion referencing "bar"
        //  2) The "just/conf" placeholders for "bar"
        assert_eq!(expansions.len(), 2, "Should have exactly two expansions for one field");

        // The doc line should be included in the child's schema literal
        // (the function typically transforms doc lines into a doc_lit).
        let merged = expansions_str.join("\n");
        assert!(
            merged.contains("bar") && merged.contains("A doc line for bar"),
            "Should reference 'bar' and doc 'A doc line for bar'"
        );
        assert!(
            merged.contains("bar_justification") && merged.contains("bar_confidence"),
            "Should contain placeholders for bar justification/confidence"
        );
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_justify_false() {
        trace!("Starting test_gather_schemas_and_placeholders_justify_false");
        let src = r#"
            struct JustifyFalseTest {
                /// some doc
                #[justify = false]
                alpha: String,

                /// another doc
                beta: i32
            }
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        let expansions_str = tokens_to_string_vec(&expansions);
        debug!("Expansions: {:#?}", expansions_str);

        // We expect expansions for each field's child schema, but placeholders only for `beta`.
        // Because `alpha` has `#[justify = false]`.
        // So expansions should be:
        //   - alpha child schema
        //   - beta child schema
        //   - placeholders for beta (NOT alpha)
        //
        // That totals 3 expansions, since alpha won't have the just/conf placeholders.
        assert_eq!(expansions.len(), 3, "We expect 3 expansions total");

        // expansions[0] is alpha child schema
        // expansions[1] is beta child schema
        // expansions[2] is placeholders for beta
        let joined = expansions_str.join("\n");

        // Check that alpha was inserted
        assert!(joined.contains("map.insert(\"alpha\""), "alpha child schema");
        // Check that beta was inserted
        assert!(joined.contains("map.insert(\"beta\""), "beta child schema");

        // Only beta should have placeholders
        assert!(
            !joined.contains("alpha_justification"),
            "alpha should not have justification placeholders"
        );
        assert!(
            joined.contains("beta_justification"),
            "beta should have justification placeholders"
        );
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_justify_inner_false() {
        trace!("Starting test_gather_schemas_and_placeholders_justify_inner_false");
        let src = r#"
            struct JustifyInnerFalseTest {
                /// Outer doc
                #[doc="with multiple lines"]
                #[justify_inner = false]
                gamma: Vec<u32>,

                delta: bool
            }
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        let expansions_str = tokens_to_string_vec(&expansions);
        debug!("Expansions: {:#?}", expansions_str);

        // The function doesn't skip top-level placeholders if `justify` is not false,
        // but it might skip child expansions if `#[justify_inner=false]` is found.
        // Actually, gather_schemas_and_placeholders_for_named_fields creates a child's schema
        // plus placeholders for *this* field if `justify=false` isn't present.
        // But skip_child_just is a deeper logic. We'll just ensure expansions exist
        // and that gamma has placeholders because we didn't say `justify=false`, only `justify_inner=false`.
        //
        // So expansions:
        //   1) gamma child schema
        //   2) gamma placeholders
        //   3) delta child schema
        //   4) delta placeholders
        //
        // That means 4 expansions total, unless there's some nuance with skip_child_just.
        // We'll just check the expansions have a consistent shape.

        assert_eq!(expansions.len(), 4, "Expected 4 expansions for 2 fields with placeholders each");

        let joined = expansions_str.join("\n");
        assert!(joined.contains("gamma"), "Gamma expansions must appear");
        assert!(joined.contains("delta"), "Delta expansions must appear");
        assert!(joined.contains("delta_justification"), "Delta placeholders must appear");
        assert!(joined.contains("gamma_justification"), "Gamma placeholders must appear");
        // Because it's not `#[justify=false]`, we do want gamma placeholders.
        // The `justify_inner=false]` would skip deeper expansions inside a child type,
        // but gather_schemas_and_placeholders_for_named_fields still inserts the top-level placeholders.
    }

    #[traced_test]
    fn test_gather_schemas_and_placeholders_multiple_fields() {
        trace!("Starting test_gather_schemas_and_placeholders_multiple_fields");
        let src = r#"
            struct ComplexFieldsTest {
                /// doc for foo
                foo: String,

                /// doc for bar
                #[justify = false]
                bar: i64,

                baz: Option<bool>
            }
        "#;
        let fields = parse_named_fields(src);
        let expansions = gather_schemas_and_placeholders_for_named_fields(&fields);
        let expansions_str = tokens_to_string_vec(&expansions);
        debug!("Expansions: {:#?}", expansions_str);

        // We have 3 fields: foo, bar, baz.
        // The function will produce:
        //   - Child schema for foo
        //   - Just/conf placeholders for foo
        //   - Child schema for bar
        //   (No placeholders for bar => justify = false)
        //   - Child schema for baz
        //   - Just/conf placeholders for baz (since it doesn't have justify=false)
        //
        // => total expansions = 1+1 + 1 + 1+1 = 5
        // Actually that's 1 + 1 (foo) + 1 (bar only child) + 1 + 1 (baz) = 5 expansions
        assert_eq!(expansions.len(), 5, "Expected 5 expansions total");

        let joined = expansions_str.join("\n");
        // foo expansions
        assert!(joined.contains("map.insert(\"foo\""), "foo child schema");
        assert!(joined.contains("foo_justification") && joined.contains("foo_confidence"));

        // bar expansions
        assert!(joined.contains("map.insert(\"bar\""), "bar child schema");
        assert!(!joined.contains("bar_justification"), "no placeholders for bar");

        // baz expansions
        assert!(joined.contains("map.insert(\"baz\""), "baz child schema");
        assert!(joined.contains("baz_justification") && joined.contains("baz_confidence"));
    }
}
