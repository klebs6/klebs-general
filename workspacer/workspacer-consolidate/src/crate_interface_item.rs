// ---------------- [ File: src/crate_interface_item.rs ]
crate::ix!();

/// Holds a single top-level item (fn, struct, etc.)
#[derive(Getters, Debug, Clone)]
#[getset(get="pub")]
pub struct CrateInterfaceItem<T: GenerateSignature> {
    item:        Arc<T>,
    docs:        Option<String>,
    attributes:  Option<String>,
    body_source: Option<String>,
}

unsafe impl<T: GenerateSignature> Send for CrateInterfaceItem<T> {}
unsafe impl<T: GenerateSignature> Sync for CrateInterfaceItem<T> {}

impl<T: GenerateSignature> CrateInterfaceItem<T> {

    pub fn new(
        item: T,
        docs:        Option<String>,
        attributes:  Option<String>,
        body_source: Option<String>,

    ) -> Self {

        // unify doc lines from base_docs + #[doc="..."] attributes
        let final_docs = merge_doc_attrs(docs, &attributes);

        // skip doc lines (#[doc=...], #![doc=...], or lines starting with ///) from attributes
        let filtered_attrs = attributes.map(|txt| {
            txt.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    // skip if it's a doc attribute or a `///`
                    !(trimmed.starts_with("#[doc =")
                      || trimmed.starts_with("#![doc =")
                      || trimmed.starts_with("///"))
                })
                .collect::<Vec<_>>()
                .join("\n")
        });

        Self {
            item: Arc::new(item),
            docs: final_docs,
            attributes: filtered_attrs,
            body_source,
        }
    }
}

impl<T: GenerateSignature> fmt::Display for CrateInterfaceItem<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1) Doc lines first
        if let Some(docs) = &self.docs {
            for line in docs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 2) Then attributes
        if let Some(attrs) = &self.attributes {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 3) Then the signature
        let signature = self.item.generate_signature();
        write!(f, "{}", signature)?;

        // 4) If it's a function, handle the body
        if signature.contains("fn ") {
            if let Some(ref body_text) = self.body_source {
                // parse lines, remove leading/trailing braces, re-indent
                let lines: Vec<_> = body_text.lines().map(|l| l.to_string()).collect();
                if lines.is_empty() {
                    writeln!(f, " {{ /* ... */ }}")?;
                } else {
                    let mut content_lines = lines.clone();
                    if content_lines.first().map(|s| s.trim()) == Some("{") {
                        content_lines.remove(0);
                    }
                    if content_lines.last().map(|s| s.trim()) == Some("}") {
                        content_lines.pop();
                    }

                    writeln!(f, " {{")?;
                    let min_indent = content_lines
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| leading_spaces(l))
                        .min()
                        .unwrap_or(0);

                    for line in content_lines {
                        let trimmed = if line.trim().is_empty() {
                            "".to_string()
                        } else {
                            line.chars().skip(min_indent).collect::<String>()
                        };
                        writeln!(f, "    {}", trimmed)?;
                    }

                    writeln!(f, "}}")?;
                }
            } else {
                writeln!(f, " {{ /* ... */ }}")?;
            }
        } else {
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[disable]
mod test_crate_interface_item {
    use super::*;
    use std::sync::Arc;

    // A minimal T that implements `GenerateSignature` for testing.
    // We'll let it store a signature string (e.g. "fn foo()" or "struct Bar").
    #[derive(Debug, Clone)]
    struct MockItem {
        signature: String,
    }

    // We'll implement the required trait:
    impl GenerateSignature for MockItem {
        fn generate_signature(&self) -> String {
            self.signature.clone()
        }
    }

    // ------------------------------------------------------------------------
    // Test Cases for CrateInterfaceItem<MockItem>
    // ------------------------------------------------------------------------

    /// 1) If we have no docs, no attributes, no body, we confirm that
    ///    `CrateInterfaceItem::new` leaves docs/attributes/body as None and display is minimal.
    #[test]
    fn test_no_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn no_docs_or_attrs()".to_string(),
        };
        // Construct the item
        let ci = CrateInterfaceItem::new(mock, None, None, None);

        // Check getters
        assert_eq!(ci.docs(), None, "No docs => None");
        assert_eq!(ci.attributes(), None, "No attrs => None");
        assert_eq!(ci.body_source(), None, "No body => None");
        // The item is Arc<MockItem>. We can check the signature
        assert_eq!(
            ci.item().generate_signature(),
            "fn no_docs_or_attrs()",
            "Signature from mock"
        );

        // Check Display output
        // We expect something like:
        //   fn no_docs_or_attrs() {
        //       /* ... */
        //   }
        // Because the code does `if signature.contains("fn ")` => print body braces
        let display_str = format!("{}", ci);
        // We expect:
        //   fn no_docs_or_attrs() { /* ... */ }
        assert!(display_str.contains("fn no_docs_or_attrs()"));
        assert!(display_str.contains("{ /* ... */ }"));
    }

    /// 2) If we have doc lines but no attributes, no body => the doc lines appear first, then signature, then { /* ... */ } if it's a fn.
    #[test]
    fn test_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn doc_test()".to_string(),
        };
        let docs = Some("/// Doc line one\n/// Doc line two".to_string());

        let ci = CrateInterfaceItem::new(mock, docs.clone(), None, None);

        // Check merged docs => we have no attribute-based docs to merge, so final is the same
        assert!(ci.docs().is_some());
        let final_docs = ci.docs().unwrap();
        assert!(final_docs.contains("/// Doc line one"));
        assert!(final_docs.contains("/// Doc line two"));

        // No attributes => None
        assert_eq!(ci.attributes(), None);
        // No body => None
        assert_eq!(ci.body_source(), None);

        // Check Display
        let display_str = format!("{}", ci);
        // Expect doc lines, each on its own line, then signature, then { /* ... */ }.
        let lines: Vec<_> = display_str.lines().collect();
        // Something like:
        //   /// Doc line one
        //   /// Doc line two
        //   fn doc_test() { /* ... */ }
        assert_eq!(lines[0], "/// Doc line one");
        assert_eq!(lines[1], "/// Doc line two");
        assert!(lines.last().unwrap().contains("fn doc_test()"));
    }

    /// 3) If we have doc lines plus attribute lines (including doc attributes), we test the "merge_doc_attrs" logic.
    ///    doc attributes lines get merged into docs, normal attributes remain in `attributes`.
    #[test]
    fn test_merge_doc_attrs_with_attributes() {
        let mock = MockItem {
            signature: "fn attr_merge()".to_string(),
        };
        let base_docs = Some("/// existing doc line".to_string());
        let raw_attrs = Some(
            "#[doc = \"another doc line\"]\n#[inline]\n#[doc = \"yet another line\"]\n#[feature(xyz)]\n/// some inline doc"
                .to_string(),
        );
        // Observations:
        // - lines starting with `#[doc = ...]` => turned into `/// ...` lines and merged into docs
        // - lines with `#[inline]` or `#[feature(xyz)]` remain in attributes
        // - lines with `/// some inline doc` also merges into docs if recognized

        let ci = CrateInterfaceItem::new(mock, base_docs, raw_attrs, None);

        // Check final docs => should combine "existing doc line" with two doc attribute lines + the triple-slash doc line
        let final_docs_opt = ci.docs();
        assert!(final_docs_opt.is_some(), "We should have final docs after merging");
        let final_docs = final_docs_opt.unwrap();
        // Should contain "/// existing doc line", "/// another doc line", "/// yet another line", and "/// some inline doc"
        assert!(final_docs.contains("/// existing doc line"));
        assert!(final_docs.contains("/// another doc line"));
        assert!(final_docs.contains("/// yet another line"));
        assert!(final_docs.contains("/// some inline doc"));

        // Check final attributes => should keep "#[inline]" and "#[feature(xyz)]" but skip doc lines
        let final_attrs_opt = ci.attributes();
        assert!(final_attrs_opt.is_some());
        let final_attrs = final_attrs_opt.unwrap();
        // Expect 2 lines
        let lines: Vec<_> = final_attrs.lines().collect();
        assert_eq!(lines.len(), 2, "We keep exactly 2 lines of attributes");
        assert!(lines[0].contains("#[inline]"));
        assert!(lines[1].contains("#[feature(xyz)]"));

        // No body => None
        assert_eq!(ci.body_source(), None);

        // Check Display
        let display_str = format!("{}", ci);
        // The doc lines appear first, then the 2 attributes, then `fn attr_merge() { /* ... */ }`.
        // We'll do a partial check:
        assert!(display_str.contains("/// existing doc line"));
        assert!(display_str.contains("#[inline]"));
        assert!(display_str.contains("#[feature(xyz)]"));
        assert!(display_str.contains("fn attr_merge()"));
    }

    /// 4) If the signature is something else (like "struct Foo"), then the display doesn't insert a body (because the code checks `if signature.contains("fn ")`).
    #[test]
    fn test_non_function_signature_struct() {
        let mock = MockItem {
            signature: "struct Foo".to_string(),
        };
        let docs = Some("/// doc for Foo".to_string());
        let attrs = Some("#[derive(Debug)]".to_string());
        let ci = CrateInterfaceItem::new(mock, docs.clone(), attrs.clone(), None);

        // Check that no body is used, since "struct Foo" doesn't contain "fn "
        let display_str = format!("{}", ci);
        // Expect:
        //   /// doc for Foo
        //   #[derive(Debug)]
        //   struct Foo
        // <and a newline after that, no braces>
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "/// doc for Foo");
        assert_eq!(lines[1], "#[derive(Debug)]");
        assert_eq!(lines[2], "struct Foo");
        assert_eq!(lines.len(), 3, "No extra braces or body");
    }

    /// 5) If we have a body_source for a function, the display re-indents it between braces. 
    ///    We'll feed a multiline block, ensuring it handles leading/trailing braces removal and indentation.
    #[test]
    fn test_fn_body_display_multiline_block() {
        let mock = MockItem {
            signature: "fn multiline()".to_string(),
        };
        let docs = None;
        let attrs = None;
        let body_source = Some(
            r#"{
    let x = 10;
    println!("x = {}", x);
}"#
            .to_string(),
        );

        let ci = CrateInterfaceItem::new(mock, docs, attrs, body_source.clone());

        // The code removes the first/last brace lines and re-indents by 4 spaces inside the final output.
        let display_str = format!("{}", ci);
        // Expect something like:
        //   fn multiline() {
        //       let x = 10;
        //       println!("x = {}", x);
        //   }
        assert!(display_str.contains("fn multiline() {"));
        assert!(display_str.contains("let x = 10;"));
        assert!(display_str.contains("println!(\"x = {}\", x);"));
        assert!(display_str.contains("}")); // final brace
    }

    /// 6) If body_source is empty or just "{}", we get a single line `{ /* ... */ }`.
    #[test]
    fn test_empty_body_source() {
        let mock = MockItem {
            signature: "fn empty_body()".to_string(),
        };
        // We'll define an empty block or none
        let body_source = Some("{}".to_string());

        let ci = CrateInterfaceItem::new(mock, None, None, body_source);
        let display_str = format!("{}", ci);

        // Should see "fn empty_body() { /* ... */ }"
        assert!(display_str.contains("fn empty_body() { /* ... */ }"));
    }

    /// 7) If body_source is None, we also get `{ /* ... */ }`.
    #[test]
    fn test_no_body_source() {
        let mock = MockItem {
            signature: "fn no_body_source()".to_string(),
        };
        let ci = CrateInterfaceItem::new(mock, None, None, None);
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn no_body_source() { /* ... */ }"));
    }

    /// 8) We can pass a large doc string or multiple lines. We'll confirm each line is printed. 
    ///    Also confirm the doc lines appear before attributes and signature.
    #[test]
    fn test_large_docs_multiple_lines() {
        let mock = MockItem {
            signature: "fn multi_line_docs()".to_string(),
        };
        let docs = Some(
            r#"/// line one
/// line two
/// line three"#
                .to_string(),
        );
        let attrs = Some("#[test_attr]".to_string());
        let ci = CrateInterfaceItem::new(mock, docs.clone(), attrs.clone(), None);

        let display_str = format!("{}", ci);
        let lines: Vec<_> = display_str.lines().collect();
        // Expect the doc lines first, each on its own line, then attribute, then signature + { /* ... */ } 
        assert_eq!(lines[0], "/// line one");
        assert_eq!(lines[1], "/// line two");
        assert_eq!(lines[2], "/// line three");
        assert_eq!(lines[3], "#[test_attr]");
        assert!(lines[4].contains("fn multi_line_docs()"));
    }

    /// 9) If the item signature doesn't contain "fn ", but is e.g. "impl Trait for Type", we skip the body even if body_source is Some.
    #[test]
    fn test_item_not_fn_but_has_body_source() {
        let mock = MockItem {
            signature: "impl MyTrait for MyType".to_string(),
        };
        let ci = CrateInterfaceItem::new(
            mock,
            Some("// doc for impl".to_string()),
            Some("#[some_attr]".to_string()),
            Some("{ let x = 10; }".to_string()), // We'll ignore this since it's not "fn "
        );
        let display_str = format!("{}", ci);
        // We expect:
        //   // doc for impl
        //   #[some_attr]
        //   impl MyTrait for MyType
        // (and a newline, no body braces)
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "// doc for impl");
        assert_eq!(lines[1], "#[some_attr]");
        assert_eq!(lines[2], "impl MyTrait for MyType");
        assert_eq!(lines.len(), 3, "No braces or body after that");
    }

    /// 10) A more complex scenario mixing doc attributes, normal doc lines, multiple attributes, 
    ///     non-empty function body, to confirm everything merges, filters, and displays properly.
    #[test]
    fn test_complex_scenario() {
        let mock = MockItem {
            signature: "fn complex()".to_string(),
        };
        let docs = Some(
            r#"/// doc line from code
/// another doc line
"#
            .to_string(),
        );
        let raw_attrs = Some(
            r#"#[doc = "doc from attr"]
#[inline]
#[doc="another doc from attr"]
#[other_attr]
/// doc from attr that should also merge
"#
            .to_string(),
        );
        // We define a multi-line body 
        let body_source = Some(
            r#"{
    let value = 42;
    println!("value = {}", value);
}"#
            .to_string(),
        );

        let ci = CrateInterfaceItem::new(mock, docs, raw_attrs, body_source);

        // 1) Merged docs => code doc lines plus doc= lines plus triple-slash doc lines 
        let merged_docs = ci.docs().as_ref().expect("Should have final merged docs");
        assert!(merged_docs.contains("/// doc line from code"));
        assert!(merged_docs.contains("/// another doc line"));
        // from attr
        assert!(merged_docs.contains("/// doc from attr"));
        assert!(merged_docs.contains("/// another doc from attr"));
        assert!(merged_docs.contains("/// doc from attr that should also merge"));

        // 2) Filtered attributes => keep inline, other_attr, but skip doc lines
        let final_attrs = ci.attributes().as_ref().expect("Should have final attributes after filtering doc lines");
        let attr_lines: Vec<_> = final_attrs.lines().collect();
        assert_eq!(attr_lines.len(), 2, "We expect 2 normal attributes, inline + other_attr");
        assert!(attr_lines[0].contains("#[inline]"));
        assert!(attr_lines[1].contains("#[other_attr]"));

        // 3) The body_source => multi-line block. The display should re-indent it inside braces.
        let display_str = format!("{}", ci);
        // Some checks
        assert!(display_str.contains("fn complex() {"));
        assert!(display_str.contains("let value = 42;"));
        assert!(display_str.contains("println!(\"value = {}\", value);"));
        assert!(display_str.contains("}"));
    }
}
