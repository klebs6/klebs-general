// ---------------- [ File: ai-json-template-derive/src/gather_schemas_and_placeholders_for_named_fields.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn gather_schemas_and_placeholders_for_named_fields(
    fields: &syn::FieldsNamed
) -> Vec<proc_macro2::TokenStream>
{
    let mut expansions = Vec::new();

    for field in &fields.named {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                // Should never happen for named fields, but just in case:
                debug!("Skipping unnamed field in a named struct?");
                continue;
            }
        };
        let field_name_str = field_ident.to_string();

        // Gather doc lines
        let doc_str = gather_doc_comments(&field.attrs).join("\n");

        // required = (not an Option<...>)
        let is_required = extract_option_inner(&field.ty).is_none();

        // Evaluate justification attributes
        let skip_f_self  = is_justification_disabled_for_field(field);
        let skip_f_inner = is_justification_disabled_for_inner(field);

        trace!(
            "Processing field '{}' => skip_f_self={}, skip_f_inner={}",
            field_name_str, skip_f_self, skip_f_inner
        );

        // (1) Build the child schema. If skip_f_inner=true, we skip any *nested*
        //     expansions (like justification for the child’s subfields),
        //     but we do still produce a "leaf" schema. So we pass skip_child_just=skip_f_inner
        //     to `build_named_field_child_schema_expr`.
        if let Some(child_expr) = build_named_field_child_schema_expr(
            field,
            &doc_str,
            is_required,
            /* skip_child_just = */ skip_f_inner
        ) {
            expansions.push(quote::quote! {
                map.insert(#field_name_str.to_string(), #child_expr);
            });
        }

        // (2) Insert top-level placeholders unless `#[justify=false]`.
        //     We do *not* skip placeholders for `#[justify_inner=false]` in a struct,
        //     because the test suite wants them. So only skip if skip_f_self==true.
        if !skip_f_self {
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
            expansions_str[0].contains("map . insert (\"foo\""),
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
        info!("joined={}",joined);

        // Check that alpha was inserted
        assert!(joined.contains("map . insert (\"alpha\""), "alpha child schema");
        // Check that beta was inserted
        assert!(joined.contains("map . insert (\"beta\""), "beta child schema");

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

    /// A small helper that takes a token stream (the expansion),
    /// wraps it in braces, and parses as a syn::Block.
    /// That way, we get zero or more statements we can match on.
    fn parse_as_statements(ts: proc_macro2::TokenStream) -> syn::Result<Vec<Stmt>> {
        let wrapped = quote::quote! {
            {
                #ts
            }
        };
        let block: Block = parse2(wrapped)?;
        Ok(block.stmts)
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
        // We expect 5 expansions:
        //   (1) child schema for foo
        //   (2) placeholders for foo
        //   (3) child schema for bar
        //   (4) child schema for baz
        //   (5) placeholders for baz
        assert_eq!(expansions.len(), 5, "Expected 5 expansions total");

        // --- (1) Child schema for `foo`
        {
            let stmts = parse_as_statements(expansions[0].clone())
                .expect("Should parse as statements");
            // Typically it's a single statement: `map.insert("foo", { ... });`
            assert_eq!(stmts.len(), 1, "Expected exactly 1 statement for foo child schema");

            match &stmts[0] {
                Stmt::Expr(Expr::MethodCall(mc), _) => {
                    // Something like: map . insert("foo", <expr>)
                    // Check the method name is "insert"
                    assert_eq!(mc.method.to_string(), "insert");
                    // Check the receiver is a path "map"
                    if let Expr::Path(ref p) = *mc.receiver {
                        assert_eq!(p.path.segments[0].ident, "map");
                    } else {
                        panic!("Expected receiver to be `map`");
                    }
                    // Optionally, check first argument is `"foo".to_string()` or just `"foo"`.
                    // Because you might do `foo.to_string()` or something else. 
                    // For simplicity, let’s just check we see "foo" somewhere in the argument tokens:
                    let arg0_str = mc.args[0].to_token_stream().to_string();
                    assert!(
                        arg0_str.contains("\"foo\""),
                        "First arg should mention \"foo\""
                    );
                }
                _ => panic!("Expected `map.insert(\"foo\", ...)` as a single statement for foo"),
            }
        }

        // --- (2) Placeholders for `foo`
        {
            let stmts = parse_as_statements(expansions[1].clone())
                .expect("Should parse the placeholders for foo");
            // We expect a block with ~2 statements inside: one for foo_justification, one for foo_confidence.
            // This might be a single block or multiple statements. Let’s at least check we see "foo_justification".
            let joined = stmts.iter().map(|s| s.to_token_stream().to_string()).collect::<String>();
            assert!(joined.contains("foo_justification"), "Should define foo_justification placeholder");
            assert!(joined.contains("foo_confidence"), "Should define foo_confidence placeholder");
        }

        // --- (3) Child schema for `bar` (because justify=false => no placeholders)
        {
            let stmts = parse_as_statements(expansions[2].clone())
                .expect("Should parse as statements");
            assert_eq!(stmts.len(), 1, "Expected exactly 1 statement for bar child schema");
            // We can do the same approach: confirm method call is `map.insert("bar", ...)`
            match &stmts[0] {
                Stmt::Expr(Expr::MethodCall(mc), _) => {
                    assert_eq!(mc.method.to_string(), "insert");
                    let arg0_str = mc.args[0].to_token_stream().to_string();
                    assert!(arg0_str.contains("\"bar\""), "Should mention \"bar\" as first arg");
                }
                _ => panic!("Expected `map.insert(\"bar\", ...)` as a single statement for bar"),
            }
        }

        // --- (4) Child schema for `baz`
        {
            let stmts = parse_as_statements(expansions[3].clone())
                .expect("Should parse as statements for baz child schema");
            assert_eq!(
                stmts.len(),
                1,
                "Expected a single statement for baz child schema"
            );
            let joined = stmts[0].to_token_stream().to_string();
            assert!(joined.contains("\"baz\""), "Should mention \"baz\" as first arg");
        }

        // --- (5) Placeholders for `baz`
        {
            let stmts = parse_as_statements(expansions[4].clone())
                .expect("Should parse as statements for baz placeholders");
            let joined = stmts.iter().map(|s| s.to_token_stream().to_string()).collect::<String>();
            assert!(joined.contains("baz_justification"), "Should define baz_justification");
            assert!(joined.contains("baz_confidence"), "Should define baz_confidence");
        }
    }
}
