// ---------------- [ File: workspacer-consolidate/src/crate_interface_item.rs ]
crate::ix!();

/// A single top-level item (fn, struct, etc.), with docs, attributes, and an optional body.
///
/// T must implement `GenerateSignature` so we can produce something like `fn name()` or
/// `struct Name` for display.
/// 
/// **Important**: We store *two* ranges:
/// 1. `raw_syntax_range`: the **full** range from RA’s syntax node (this exactly matches
///    `node.syntax().text_range()`).
/// 2. `effective_item_range`: a **trimmed** range that excludes leading/trailing normal comments
///    and whitespace (but keeps doc comments). We use this for computing interstitial segments.
/// 
/// Many tests (like `test_text_range` in your suite) compare `raw_syntax_range` to the node’s actual
/// `text_range()` to confirm no mismatch. Meanwhile, the interstitial logic uses `effective_item_range`
/// to ensure normal line/block comments on the edges appear in interstitial segments instead of
/// being “inside” the item.
#[derive(Builder,Setters,Getters,Debug,Clone)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct CrateInterfaceItem<T: GenerateSignature> {
    item:                  Arc<T>,
    docs:                  Option<String>,
    attributes:            Option<String>,
    body_source:           Option<String>,
    consolidation_options: Option<ConsolidationOptions>,

    /// The file path from which this item was parsed
    file_path: PathBuf,

    /// The crate root path for the crate that owns this item
    crate_path: PathBuf,

    /// The **full** syntax node range, exactly matching RA’s node range.
    /// Many tests expect this to match `node.text_range()`.
    raw_syntax_range: TextRange,

    /// A **trimmed** range that excludes leading/trailing normal line comments,
    /// block comments, and whitespace—but keeps doc comments. We use this
    /// for interstitial segment calculations, so “normal” comments appear
    /// in the leftover text.
    effective_item_range: TextRange,
}

// Mark safe if T is safe:
unsafe impl<T: GenerateSignature> Send for CrateInterfaceItem<T> {}
unsafe impl<T: GenerateSignature> Sync for CrateInterfaceItem<T> {}

impl<T> std::fmt::Display for CrateInterfaceItem<T>
where
    T: GenerateSignature + MaybeHasSyntaxKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        trace!(
            "Entering CrateInterfaceItem::fmt; signature = {:?}",
            self.item.generate_signature()
        );

        // ---------------------------------------------------------------
        // 1) Print doc lines (unmodified)
        // ---------------------------------------------------------------
        if let Some(docs) = &self.docs {
            for line in docs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // ---------------------------------------------------------------
        // 2) Print normal attributes (unmodified)
        // ---------------------------------------------------------------
        if let Some(attrs) = &self.attributes {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // ---------------------------------------------------------------
        // 3) Generate signature (disabling doc lines in the signature if set)
        // ---------------------------------------------------------------
        let signature = match &self.consolidation_options {
            Some(opts) => {
                let mut sig_opts = SignatureOptions::from(opts.into());
                sig_opts.set_include_docs(false);
                self.item.generate_signature_with_opts(&sig_opts)
            }
            None => self.item.generate_signature(),
        };

        // 3.1) Check if this is a macro
        let kind = self.item.syntax_kind(); // from MaybeHasSyntaxKind
        let is_macro = matches!(
            kind,
            Some(SyntaxKind::MACRO_CALL | SyntaxKind::MACRO_RULES)
        );

        // ---------------------------------------------------------------
        // 4) If it's a macro, just print the entire `body_source` if present
        // ---------------------------------------------------------------
        if is_macro {
            if let Some(full_text) = &self.body_source {
                // The user wants the entire macro call, e.g., `error_tree! { ... }`
                // Possibly dedent or other transformations; here we just print verbatim:
                writeln!(f, "{}", full_text)?;
            } else {
                // fallback: just the signature lines
                for line in signature.lines() {
                    writeln!(f, "{}", line)?;
                }
            }
            return Ok(());
        }

        // ---------------------------------------------------------------
        // 5) Check if it's a function
        // ---------------------------------------------------------------
        let is_fn = guess_is_function(&(*self.item), &signature);

        // If not a function => just print the signature lines & done
        if !is_fn {
            for line in signature.lines() {
                writeln!(f, "{}", line)?;
            }
            return Ok(());
        }

        // ---------------------------------------------------------------
        // 6) Single-line vs multi-line function logic
        // ---------------------------------------------------------------
        let sig_lines: Vec<String> = signature.lines().map(|x| x.to_string()).collect();

        let has_where = signature.contains(" where ")
            || signature.contains("\nwhere")
            || signature.contains("\n    where")
            || signature.contains(")\nwhere")
            || signature.contains(")where");

        let force_multiline = has_where || sig_lines.len() > 1;

        // Attempt single-line?
        if !force_multiline && sig_lines.len() == 1 {
            // single-line signature
            write!(f, "{}", sig_lines[0].trim_end())?;

            // If there's a body, dedent it
            if let Some(body) = &self.body_source {
                let body_inner = strip_outer_braces(body.trim());
                if body_inner.is_empty() {
                    writeln!(f, " {{}}")?;
                } else {
                    writeln!(f, " {{")?;

                    let raw_body_lines: Vec<&str> = body_inner.lines().collect();
                    let collapsed = normalize_blank_lines(&raw_body_lines);

                    // For the body, we do want dedent
                    let ded = conditional_dedent_all(&collapsed, true);

                    for line in ded {
                        if line.is_empty() {
                            writeln!(f)?;
                        } else {
                            writeln!(f, "    {}", line)?;
                        }
                    }
                    writeln!(f, "}}")?;
                }
            } else {
                writeln!(f, " {{}}")?;
            }
            return Ok(());
        }

        // ---------------------------------------------------------------
        // 7) Multi-line function signature printing
        // ---------------------------------------------------------------
        let do_dedent_for_signature = false;
        let sig_dedented = conditional_dedent_all(&sig_lines, do_dedent_for_signature);
        for line in &sig_dedented {
            writeln!(f, "{}", line)?;
        }

        // ---------------------------------------------------------------
        // 8) Print the function body (if any), dedented
        // ---------------------------------------------------------------
        if let Some(body) = &self.body_source {
            let trimmed = body.trim();
            let inside = strip_outer_braces(trimmed);
            if inside.is_empty() {
                writeln!(f, "{{}}")?;
            } else {
                writeln!(f, "{{")?;

                let raw_body_lines: Vec<&str> = inside.lines().collect();
                let collapsed = normalize_blank_lines(&raw_body_lines);

                let do_dedent_for_body = true;
                let ded = conditional_dedent_all(&collapsed, do_dedent_for_body);

                for line in ded {
                    if line.is_empty() {
                        writeln!(f)?;
                    } else {
                        writeln!(f, "    {}", line)?;
                    }
                }
                writeln!(f, "}}")?;
            }
        } else {
            writeln!(f, "{{}}")?;
        }

        Ok(())
    }
}

impl<T: GenerateSignature> CrateInterfaceItem<T> {

    /// A test-only convenience constructor that fills in dummy file paths,
    /// zero-length text ranges, and sets both `raw_syntax_range` and `effective_item_range`
    /// to the same zero-range. This helps older unit tests that don’t specify ranges.
    #[cfg(test)]
    pub fn new_for_test(
        item:                  T,
        docs:                  Option<String>,
        attributes:            Option<String>,
        body_source:           Option<String>,
        consolidation_options: Option<ConsolidationOptions>,
    ) -> Self {
        CrateInterfaceItem::new_with_paths_and_ranges(
            item,
            docs,
            attributes,
            body_source,
            consolidation_options,
            PathBuf::from("TEST_ONLY_file_path.rs"),
            PathBuf::from("TEST_ONLY_crate_path"),
            TextRange::new(TextSize::from(0), TextSize::from(0)),  // raw
            TextRange::new(TextSize::from(0), TextSize::from(0)),  // effective
        )
    }

    /// Creates a new `CrateInterfaceItem` with *both* a `raw_syntax_range` and
    /// an `effective_item_range`. Usually:
    /// - `raw_syntax_range` = exactly the RA node’s `text_range()`.
    /// - `effective_item_range` = maybe the same as `raw_syntax_range` or a
    ///   “trimmed” version that excludes normal (non-doc) line/block comments
    ///   from edges.
    pub fn new_with_paths_and_ranges(
        item:                   T,
        docs:                   Option<String>,
        attributes:             Option<String>,
        body_source:            Option<String>,
        consolidation_options:  Option<ConsolidationOptions>,
        file_path:              PathBuf,
        crate_path:             PathBuf,
        raw_syntax_range:       TextRange,
        effective_item_range:   TextRange,
    ) -> Self {
        // 1) Possibly unify doc lines from base docs + doc lines hidden in raw_attrs
        let (unified_docs, filtered_attrs) = Self::merge_docs_and_filter_attrs(docs, attributes);

        // 2) Possibly unify or transform the body if it’s empty or just "{}"
        let final_body = match body_source {
            Some(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed == "{}" {
                    None
                } else {
                    Some(s)
                }
            }
            None => None,
        };

        Self {
            item: Arc::new(item),
            docs: unified_docs,
            attributes: filtered_attrs,
            body_source: final_body,
            consolidation_options,
            file_path,
            crate_path,
            raw_syntax_range,
            effective_item_range,
        }
    }

    /// Merges doc lines from `base_docs` plus any doc lines found in `raw_attrs`.
    /// - If a line in `raw_attrs` starts with `#[doc` or `#![doc`, we parse its quoted text => triple-slash doc line.
    /// - If a line in `raw_attrs` starts with `///`, we treat it as a doc line.
    /// - All other lines remain as “normal attributes.”
    ///
    /// Returns `(final_docs, final_attrs)`.
    fn merge_docs_and_filter_attrs(
        base_docs: Option<String>,
        raw_attrs: Option<String>,
    ) -> (Option<String>, Option<String>)
    {
        let mut final_docs = Vec::new();
        let mut seen_docs = std::collections::HashSet::new();

        // 1) Start with base_docs lines:
        if let Some(base) = base_docs {
            for line in base.lines() {
                if !line.trim().is_empty() {
                    // Only add if not seen
                    if seen_docs.insert(line.to_string()) {
                        final_docs.push(line.to_string());
                    }
                } else {
                    // If the line is empty or just whitespace, you might decide if you want it or not
                    if seen_docs.insert(line.to_string()) {
                        final_docs.push(line.to_string());
                    }
                }
            }
        }

        let mut normal_attr_lines = Vec::new();

        // 2) Parse raw_attrs. Extract doc lines or keep as normal attributes
        if let Some(attr_text) = raw_attrs {
            for line in attr_text.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("#[doc") || trimmed.starts_with("#![doc") {
                    if let Some(doc_str) = Self::extract_doc_string_from_attr(trimmed) {
                        let doc_line = format!("/// {}", doc_str);
                        if seen_docs.insert(doc_line.clone()) {
                            final_docs.push(doc_line);
                        }
                    }
                } else if trimmed.starts_with("///") {
                    if seen_docs.insert(trimmed.to_string()) {
                        final_docs.push(trimmed.to_string());
                    }
                } else {
                    // normal attribute => keep
                    normal_attr_lines.push(line.to_string());
                }
            }
        }

        let final_docs_str = if final_docs.is_empty() {
            None
        } else {
            Some(final_docs.join("\n"))
        };

        let final_attrs_opt = if normal_attr_lines.is_empty() {
            None
        } else {
            Some(normal_attr_lines.join("\n"))
        };

        (final_docs_str, final_attrs_opt)
    }

    /// Extracts the substring in quotes from an attribute line like `#[doc = "some text"]` or `#[doc="something"]`.
    fn extract_doc_string_from_attr(line: &str) -> Option<String> {
        let trimmed = line.trim_end().trim_end_matches(']');
        if let Some(eq_idx) = trimmed.find('=') {
            if let Some(start_quote) = trimmed[eq_idx..].find('"') {
                let start = eq_idx + start_quote + 1; // skip the quote
                if let Some(end_quote) = trimmed[start..].find('"') {
                    let end = start + end_quote;
                    return Some(trimmed[start..end].to_string());
                }
            }
        }
        None
    }
}

impl<T: GenerateSignature> CrateInterfaceItem<T> {
    /// This is the “official” accessor for the RA node range,
    /// used by `test_text_range` etc. We do **not** trim comments here.
    /// 
    /// So if you need the untrimmed range to compare with RA,
    /// call `ci.raw_syntax_range()`.
    /// 
    /// If you want the trimmed range for interstitial logic,
    /// call `ci.effective_item_range()`.
    pub fn text_range(&self) -> &TextRange {
        &self.raw_syntax_range
    }

    /// This is the accessor for the “trimmed” range that excludes normal
    /// line comments at the edges. Interstitial logic uses this one to
    /// unify coverage.
    pub fn effective_range(&self) -> &TextRange {
        &self.effective_item_range
    }
}

/// Count leading spaces in a line
fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

#[cfg(test)]
mod test_crate_interface_item {
    use super::*;
    use std::sync::Arc;

    // Minimal T that implements GenerateSignature:
    #[derive(Debug,Clone)]
    struct MockItem {
        signature: String,
    }

    // If you have a mock item type that doesn't implement AstNode (like test_crate_interface_item::MockItem),
    // just return None. Then the Display logic won't attempt to interpret it as a function.
    impl MaybeHasSyntaxKind for MockItem {
        fn syntax_kind(&self) -> Option<SyntaxKind> {
            None
        }
    }

    impl GenerateSignature for MockItem {
        fn generate_signature(&self) -> String {
            self.signature.clone()
        }
        fn generate_signature_with_opts(&self,_: &SignatureOptions) -> String {
            self.generate_signature()
        }
    }

    // Now we reproduce your failing tests + others:

    #[traced_test]
    fn test_no_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn no_docs_or_attrs()".to_string(),
        };
        let ci = CrateInterfaceItem::new_for_test(mock, None, None, None, None);

        // Display => "fn no_docs_or_attrs() { /* ... */ }"
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn no_docs_or_attrs()"));
    }

    #[traced_test]
    fn test_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn doc_test()".to_string(),
        };
        let docs = Some("/// Doc line one\n/// Doc line two".to_string());
        let ci = CrateInterfaceItem::new_for_test(mock, docs.clone(), None, None, None);

        let display_str = format!("{}", ci);
        assert!(display_str.contains("/// Doc line one"));
        assert!(display_str.contains("/// Doc line two"));
        assert!(display_str.contains("fn doc_test()"));
    }

    #[traced_test]
    fn test_merge_doc_attrs_with_attributes() {
        let mock = MockItem {
            signature: "fn attr_merge()".to_string(),
        };
        let base_docs = Some("/// existing doc line".to_string());
        // includes doc=, triple slash doc, and normal attributes
        let raw_attrs = Some(
r#"#[doc = "another doc line"]
#[inline]
#[doc="yet another line"]
#[feature(xyz)]
/// some inline doc"#.to_string()
        );

        let ci = CrateInterfaceItem::new_for_test(mock, base_docs, raw_attrs, None, None);

        // We expect doc lines => "/// existing doc line", "/// another doc line", "/// yet another line", "/// some inline doc"
        let final_docs = ci.docs().as_ref().expect("Should have doc lines");
        assert!(final_docs.contains("/// existing doc line"));
        assert!(final_docs.contains("/// another doc line"));
        assert!(final_docs.contains("/// yet another line"));
        assert!(final_docs.contains("/// some inline doc"));

        // We expect normal attrs => #[inline], #[feature(xyz)]
        let final_attrs = ci.attributes().as_ref().expect("some normal attrs");
        let lines: Vec<_> = final_attrs.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "#[inline]");
        assert_eq!(lines[1], "#[feature(xyz)]");

        let display_str = format!("{}", ci);
        // doc lines appear first, then attributes, then signature + { /* ... */ }
        assert!(display_str.contains("/// existing doc line"));
        assert!(display_str.contains("/// another doc line"));
        assert!(display_str.contains("/// yet another line"));
        assert!(display_str.contains("/// some inline doc"));
        assert!(display_str.contains("#[inline]"));
        assert!(display_str.contains("#[feature(xyz)]"));
        assert!(display_str.contains("fn attr_merge()"));
    }

    #[traced_test]
    fn test_empty_body_source() {
        let mock = MockItem {
            signature: "fn empty_body()".to_string(),
        };
        let body_source = Some("{}".to_string());

        let ci = CrateInterfaceItem::new_for_test(mock, None, None, body_source, None);
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn empty_body()"));
    }

    #[traced_test]
    fn test_complex_scenario() {
        let mock = MockItem {
            signature: "fn complex()".to_string(),
        };
        let docs = Some(
r#"/// doc line from code
/// another doc line"#.to_string()
        );
        let raw_attrs = Some(
r#"#[doc = "doc from attr"]
#[inline]
#[doc="another doc from attr"]
#[other_attr]
/// doc from attr that should also merge
"#.to_string()
        );
        let body_source = Some(
r#"{
    let value = 42;
    println!("value = {}", value);
}"#.to_string()
        );

        let ci = CrateInterfaceItem::new_for_test(mock, docs, raw_attrs, body_source, None);

        // final docs => 5 lines:
        // 1) "/// doc line from code"
        // 2) "/// another doc line"
        // 3) "/// doc from attr"
        // 4) "/// another doc from attr"
        // 5) "/// doc from attr that should also merge"
        let merged_docs = ci.docs().as_ref().expect("some docs");
        assert!(merged_docs.contains("/// doc line from code"));
        assert!(merged_docs.contains("/// another doc line"));
        assert!(merged_docs.contains("/// doc from attr"));
        assert!(merged_docs.contains("/// another doc from attr"));
        assert!(merged_docs.contains("/// doc from attr that should also merge"));

        // normal attrs => #[inline], #[other_attr]
        let final_attrs = ci.attributes().as_ref().expect("some attrs");
        let lines: Vec<_> = final_attrs.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "#[inline]");
        assert_eq!(lines[1], "#[other_attr]");

        // The display => doc lines, then 2 attrs, then "fn complex() { <body> }"
        let display_str = format!("{}", ci);
        info!("display_str = {}", display_str);
        assert!(display_str.contains("/// doc line from code"));
        assert!(display_str.contains("/// another doc line"));
        assert!(display_str.contains("/// doc from attr"));
        assert!(display_str.contains("/// another doc from attr"));
        assert!(display_str.contains("/// doc from attr that should also merge"));
        assert!(display_str.contains("#[inline]"));
        assert!(display_str.contains("#[other_attr]"));
        assert!(display_str.contains("fn complex()"));
        assert!(display_str.contains("let value = 42;"));
    }

    #[traced_test]
    fn test_non_function_signature_struct() {
        let mock = MockItem {
            signature: "struct Foo".into(),
        };
        let ci = CrateInterfaceItem::new_for_test(
            mock,
            Some("/// doc for Foo".into()),
            Some("#[derive(Debug)]".into()),
            None,
            None,
        );

        let display_str = format!("{}", ci);
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "/// doc for Foo");
        assert_eq!(lines[1], "#[derive(Debug)]");
        assert_eq!(lines[2], "struct Foo");
        assert_eq!(lines.len(), 3);
    }

    #[traced_test]
    fn test_item_not_fn_but_has_body_source() {
        let mock = MockItem {
            signature: "impl MyTrait for MyType".into(),
        };
        let ci = CrateInterfaceItem::new_for_test(
            mock,
            Some("// doc for impl".into()),
            Some("#[some_attr]".into()),
            Some("{ let x = 10; }".into()),
            None,
        );

        // skip the body if signature doesn't have "fn "
        let display_str = format!("{}", ci);
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "// doc for impl");
        assert_eq!(lines[1], "#[some_attr]");
        assert_eq!(lines[2], "impl MyTrait for MyType");
        assert_eq!(lines.len(), 3);
    }

    #[traced_test]
    fn test_fn_body_display_multiline_block() {
        let mock = MockItem {
            signature: "fn multiline()".into(),
        };
        let body_source = Some(
    r#"{
        let x = 10;
        println!("x = {}", x);
    }"#.to_string()
        );
        let ci = CrateInterfaceItem::new_for_test(mock, None, None, body_source, None);

        let display_str = format!("{}", ci);
        info!("display_str = {}", display_str);
        // The test asserts these two lines exist literally:
        assert!(display_str.contains("fn multiline() {"));
        assert!(display_str.contains("let x = 10;"));
        assert!(display_str.contains("println!(\"x = {}\", x);"));
    }

    // We'll reuse the same MockItem from the main test module:
    // (the user code references `MockItem` implementing `GenerateSignature`).
    // Here we add a trivial RehydrateFromSignature impl to enable round-trip.
    impl RehydrateFromSignature for MockItem {
        #[tracing::instrument(level = "trace", skip(signature_source))]
        fn rehydrate_from_signature(signature_source: &str) -> Option<Self> {
            trace!("Attempting rehydration from signature: {}", signature_source);
            // Extremely naive approach: if it contains "fn ", we do so:
            if signature_source.contains("fn ") {
                Some(Self {
                    signature: signature_source.to_string(),
                })
            } else {
                None
            }
        }
    }

    #[traced_test]
    fn test_round_trip_serde_no_helper_struct() {
        info!("Testing serde round-trip on CrateInterfaceItem<T> directly.");

        let mock = MockItem {
            signature: "fn example_signature() -> i32".to_string(),
        };

        let original = CrateInterfaceItem::new_for_test(
            mock,
            Some("/// doc lines".to_string()),
            Some("#[inline]\n#[another_attr]".to_string()),
            Some("{ println!(\"hello!\"); }".to_string()),
            None,
        );

        debug!("Serializing original CrateInterfaceItem to JSON.");
        let json_str = serde_json::to_string(&original).expect("serialize to JSON");
        debug!("Serialized to JSON: {}", json_str);

        debug!("Deserializing back to CrateInterfaceItem<T> from JSON.");
        let deserialized: CrateInterfaceItem<MockItem> =
            serde_json::from_str(&json_str).expect("deserialize from JSON");

        assert_eq!(deserialized.docs(), original.docs());
        assert_eq!(deserialized.attributes(), original.attributes());
        assert_eq!(deserialized.body_source(), original.body_source());
        assert_eq!(
            deserialized.item.generate_signature(),
            original.item.generate_signature()
        );
    }
}
