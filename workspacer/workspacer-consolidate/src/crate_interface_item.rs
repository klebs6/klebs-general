// ---------------- [ File: src/crate_interface_item.rs ]
crate::ix!();

/// A single top-level item (fn, struct, etc.), with docs, attributes, and an optional body.
///
/// `T` must implement `GenerateSignature` so we can produce something like "fn name()" or
/// "struct Name" for display.
#[derive(Getters, Debug, Clone)]
#[getset(get="pub")]
pub struct CrateInterfaceItem<T: GenerateSignature> {
    item:        Arc<T>,
    docs:        Option<String>,
    attributes:  Option<String>,
    body_source: Option<String>,
}

// Mark safe if T is safe:
unsafe impl<T: GenerateSignature> Send for CrateInterfaceItem<T> {}
unsafe impl<T: GenerateSignature> Sync for CrateInterfaceItem<T> {}

impl<T: GenerateSignature> CrateInterfaceItem<T> {

    /// Creates a new `CrateInterfaceItem`.
    /// - `docs`: doc lines (triple-slash or so) gleaned from the code
    /// - `attributes`: raw `#[...]` lines
    /// - We unify doc lines from both `docs` and doc lines from `attributes` (e.g. `#[doc="..."]` or triple-slash lines).
    /// - We keep “normal” attributes (e.g. `#[inline]`) in `attributes`, skipping doc lines from them.
    /// - `body_source`: optional function body text. If empty or `{}`, we treat it as no real body => in Display, we show `{ /* ... */ }`.
    pub fn new(
        item: T,
        docs: Option<String>,
        attributes: Option<String>,
        body_source: Option<String>,
    ) -> Self {

        // 1) Merge doc lines from base docs + doc lines hidden in raw_attrs
        let (unified_docs, filtered_attrs) = Self::merge_docs_and_filter_attrs(docs, attributes);

        // 2) Possibly unify or transform the body if empty or just "{}"
        //    We treat that as no real body => display => `{ /* ... */ }`.
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
        }
    }

    /// Merges doc lines from `base_docs` plus any doc lines found in `raw_attrs`.
    /// - If a line in `raw_attrs` starts with `#[doc` or `#![doc`, we parse its quoted text => triple-slash doc line.
    /// - If a line in `raw_attrs` starts with `///`, we treat it as a doc line.
    /// - All other lines remain as “normal attributes.”
    ///
    /// Returns `(final_docs, final_attrs)`:
    /// - `final_docs` = Some(...) if non-empty
    /// - `final_attrs` = Some(...) if any normal attributes remain
    fn merge_docs_and_filter_attrs(
        base_docs: Option<String>,
        raw_attrs: Option<String>,
    ) -> (Option<String>, Option<String>) 
    {
        let mut final_docs = base_docs.unwrap_or_default();
        let mut normal_attr_lines = Vec::new();

        if let Some(attr_text) = raw_attrs {
            for line in attr_text.lines() {
                let trimmed = line.trim_start();

                // Check if line is doc attribute: #[doc="..."] or #![doc="..."]
                if trimmed.starts_with("#[doc")
                    || trimmed.starts_with("#![doc")
                {
                    // parse the quoted doc string => `/// <content>`
                    if let Some(doc_str) = Self::extract_doc_string_from_attr(trimmed) {
                        if !final_docs.is_empty() {
                            final_docs.push('\n');
                        }
                        final_docs.push_str("/// ");
                        final_docs.push_str(&doc_str);
                    }
                } 
                // Or if line is triple-slash doc => also add to final docs
                else if trimmed.starts_with("///") {
                    if !final_docs.is_empty() {
                        final_docs.push('\n');
                    }
                    final_docs.push_str(trimmed);
                } 
                else {
                    // normal attribute => keep
                    normal_attr_lines.push(line.to_string());
                }
            }
        }

        let final_docs_opt = if final_docs.trim().is_empty() {
            None
        } else {
            Some(final_docs)
        };

        let final_attrs_opt = if normal_attr_lines.is_empty() {
            None
        } else {
            Some(normal_attr_lines.join("\n"))
        };

        (final_docs_opt, final_attrs_opt)
    }

    /// Extracts the substring in quotes from an attribute line like `#[doc = "some text"]` or `#[doc="something"]`.
    /// Returns Some(...) if found, else None.
    fn extract_doc_string_from_attr(line: &str) -> Option<String> {
        // naive parse: find the first double-quote after '='
        // then the last double-quote before the trailing ']'.
        // We skip advanced edge cases for the tests' sake.

        // e.g. #[doc = "another doc line"]
        // or #[doc="yet another line"]
        let trimmed = line.trim_end().trim_end_matches(']');
        if let Some(eq_idx) = trimmed.find('=') {
            // look for the first quote after eq_idx
            if let Some(start_quote) = trimmed[eq_idx..].find('"') {
                let start = eq_idx + start_quote + 1; // skip the quote
                // find next quote after that
                if let Some(end_quote) = trimmed[start..].find('"') {
                    let end = start + end_quote;
                    return Some(trimmed[start..end].to_string());
                }
            }
        }
        None
    }
}

impl<T: GenerateSignature> fmt::Display for CrateInterfaceItem<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1) Print doc lines
        if let Some(docs) = &self.docs {
            for line in docs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 2) Print normal attributes
        if let Some(attrs) = &self.attributes {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 3) Signature
        let signature = self.item.generate_signature();
        write!(f, "{}", signature)?;

        // 4) If it's a function, handle body
        if signature.contains("fn ") {
            if let Some(ref body_text) = self.body_source {
                // parse lines, remove braces, re-indent
                let lines: Vec<_> = body_text.lines().map(|l| l.to_string()).collect();
                if lines.is_empty() {
                    writeln!(f, " {{ /* ... */ }}")?;
                } else {
                    let mut content_lines = lines.clone();
                    let first_trim = content_lines.first().map(|s| s.trim());
                    if first_trim == Some("{") {
                        content_lines.remove(0);
                    }
                    let last_trim = content_lines.last().map(|s| s.trim());
                    if last_trim == Some("}") {
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
                        if line.trim().is_empty() {
                            writeln!(f)?;
                        } else {
                            let reduced = line.chars().skip(min_indent).collect::<String>();
                            writeln!(f, "    {}", reduced)?;
                        }
                    }
                    writeln!(f, "}}")?;
                }
            } else {
                writeln!(f, " {{ /* ... */ }}")?;
            }
        } else {
            // not a function => newline only
            writeln!(f)?;
        }
        Ok(())
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
    impl GenerateSignature for MockItem {
        fn generate_signature(&self) -> String {
            self.signature.clone()
        }
    }

    // Now we reproduce your failing tests + others:

    #[test]
    fn test_no_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn no_docs_or_attrs()".to_string(),
        };
        let ci = CrateInterfaceItem::new(mock, None, None, None);

        // Display => "fn no_docs_or_attrs() { /* ... */ }"
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn no_docs_or_attrs() { /* ... */ }"));
    }

    #[test]
    fn test_docs_no_attrs_no_body() {
        let mock = MockItem {
            signature: "fn doc_test()".to_string(),
        };
        let docs = Some("/// Doc line one\n/// Doc line two".to_string());
        let ci = CrateInterfaceItem::new(mock, docs.clone(), None, None);

        let display_str = format!("{}", ci);
        assert!(display_str.contains("/// Doc line one"));
        assert!(display_str.contains("/// Doc line two"));
        assert!(display_str.contains("fn doc_test() { /* ... */ }"));
    }

    #[test]
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

        let ci = CrateInterfaceItem::new(mock, base_docs, raw_attrs, None);

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
        assert!(display_str.contains("fn attr_merge() { /* ... */ }"));
    }

    #[test]
    fn test_empty_body_source() {
        let mock = MockItem {
            signature: "fn empty_body()".to_string(),
        };
        let body_source = Some("{}".to_string());

        let ci = CrateInterfaceItem::new(mock, None, None, body_source);
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn empty_body() { /* ... */ }"));
    }

    #[test]
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

        let ci = CrateInterfaceItem::new(mock, docs, raw_attrs, body_source);

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

    #[test]
    fn test_non_function_signature_struct() {
        let mock = MockItem {
            signature: "struct Foo".into(),
        };
        let ci = CrateInterfaceItem::new(
            mock,
            Some("/// doc for Foo".into()),
            Some("#[derive(Debug)]".into()),
            None,
        );

        let display_str = format!("{}", ci);
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "/// doc for Foo");
        assert_eq!(lines[1], "#[derive(Debug)]");
        assert_eq!(lines[2], "struct Foo");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_item_not_fn_but_has_body_source() {
        let mock = MockItem {
            signature: "impl MyTrait for MyType".into(),
        };
        let ci = CrateInterfaceItem::new(
            mock,
            Some("// doc for impl".into()),
            Some("#[some_attr]".into()),
            Some("{ let x = 10; }".into()),
        );

        // skip the body if signature doesn't have "fn "
        let display_str = format!("{}", ci);
        let lines: Vec<_> = display_str.lines().collect();
        assert_eq!(lines[0], "// doc for impl");
        assert_eq!(lines[1], "#[some_attr]");
        assert_eq!(lines[2], "impl MyTrait for MyType");
        assert_eq!(lines.len(), 3);
    }

    #[test]
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
        let ci = CrateInterfaceItem::new(mock, None, None, body_source);
        let display_str = format!("{}", ci);
        assert!(display_str.contains("fn multiline() {"));
        assert!(display_str.contains("let x = 10;"));
        assert!(display_str.contains("println!(\"x = {}\", x);"));
    }
}
