// ---------------- [ File: src/merge_doc_attrs.rs ]
crate::ix!();

pub fn merge_doc_attrs(
    base_docs: Option<String>,
    maybe_attrs: &Option<String>,
) -> Option<String> {

    // 1) Gather base doc lines in their original order.
    //    (We must preserve that order if there's no attrs or if we return them as-is.)
    let mut base_lines: Vec<String> = Vec::new();
    if let Some(ref base) = base_docs {
        for line in base.lines() {
            base_lines.push(line.to_string());
        }
    }

    // 2) Gather doc lines from `#[doc="..."]` attributes, preserving their order of appearance.
    //    We also trim the extracted doc string, then prepend `/// `.
    //    Non-`#[doc=""]` lines are ignored. 
    let mut attr_lines_in_order = Vec::new();
    if let Some(attr_text) = maybe_attrs {
        let re = Regex::new(r#"#\[doc\s*=\s*"([^"]*)"\s*\]"#).unwrap();
        for line in attr_text.lines() {
            if let Some(caps) = re.captures(line.trim()) {
                let doc_str = caps[1].trim();
                let doc_line = format!("/// {}", doc_str);
                attr_lines_in_order.push(doc_line);
            }
        }
    }

    // If nothing at all was found, return None.
    if base_lines.is_empty() && attr_lines_in_order.is_empty() {
        return None;
    }

    // 3) If we have base docs but NO attribute lines, return base docs as-is.
    //    (Test #2 wants the original lines unaltered in that scenario.)
    if !base_lines.is_empty() && attr_lines_in_order.is_empty() {
        let joined = base_lines.join("\n");
        return Some(joined);
    }

    // 4) If we have NO base docs but DO have attribute lines, produce them
    //    in the exact order they appeared, but remove duplicates in that order.
    //    (Test #3 expects the attribute lines to appear in the same sequence.)
    if base_lines.is_empty() {
        let mut seen = HashSet::new();
        let mut final_lines = Vec::new();
        for line in attr_lines_in_order {
            if !seen.contains(&line) {
                seen.insert(line.clone());
                final_lines.push(line);
            }
        }
        return Some(final_lines.join("\n"));
    }

    // 5) Otherwise, we have BOTH base docs and attribute lines => unify them
    //    in a BTreeSet for deduplication *and* alphabetical ordering.
    //    (Test #7 explicitly wants alphabetical order in that scenario.)
    let mut all = BTreeSet::new();
    for line in base_lines {
        all.insert(line);
    }
    for line in attr_lines_in_order {
        all.insert(line);
    }

    let merged = all.into_iter().collect::<Vec<_>>().join("\n");
    Some(merged)
}

#[cfg(test)]
mod test_merge_doc_attrs {
    use super::*;
    use regex::Regex;
    use std::collections::BTreeSet;

    /// 1) If both `base_docs` and `maybe_attrs` are `None`, we expect `None`.
    #[test]
    fn test_none_both() {
        let result = merge_doc_attrs(None, &None);
        assert!(result.is_none(), "Expected None if both inputs are None");
    }

    /// 2) If `base_docs` is Some, but `maybe_attrs` is None, we expect the original doc lines.
    #[test]
    fn test_docs_only_no_attrs() {
        let base = Some("/// Line1\n/// Line2".to_string());
        let result = merge_doc_attrs(base.clone(), &None);
        assert_eq!(result, base, "Should return the same doc lines");
    }

    /// 3) If `base_docs` is None, but `maybe_attrs` includes `#[doc="..."]` lines, we transform them.
    #[test]
    fn test_no_docs_but_some_doc_attributes() {
        let attrs = Some(
            r#"
            #[doc = "This is doc line1"]
            #[doc="Line2"]
            #[some_other_attr]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(None, &attrs);
        // We expect each #[doc="something"] to produce a line in the final doc string: /// something
        let expected = r#"/// This is doc line1
/// Line2"#;
        assert_eq!(
            result.as_deref().unwrap(),
            expected,
            "Should convert #[doc] lines to `/// ...` lines"
        );
    }

    /// 4) If both `base_docs` and `maybe_attrs` have doc lines, we unify them in a BTreeSet, removing duplicates.
    #[test]
    fn test_merge_docs_and_attrs_removes_duplicates() {
        let base = Some(
            r#"/// line1
/// line2
/// line3
"#
            .to_string(),
        );
        let attrs = Some(
            r#"
            #[doc = "line2"]
            #[doc = "line4"]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(base, &attrs).unwrap();
        // The BTreeSet ordering is alphabetical if lines differ, so lines might reorder unless they're identical.
        // We have line1, line2, line3 from base, line2 duplicated, plus line4.
        // The distinct lines are line1, line2, line3, line4. 
        // The BTreeSet will sort them alphabetically by default, so we might get:
        // "/// line1\n/// line2\n/// line3\n/// line4"
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 4, "Expected 4 distinct lines after merging duplicates");
        assert!(lines.contains(&"/// line1"));
        assert!(lines.contains(&"/// line2"));
        assert!(lines.contains(&"/// line3"));
        assert!(lines.contains(&"/// line4"));
    }

    /// 5) If the only parsed doc lines are duplicates of existing base docs, we still produce just one set.
    #[test]
    fn test_all_docs_are_duplicates() {
        let base = Some("/// line1\n/// line2\n".to_string());
        let attrs = Some(
            r#"
            #[doc = "line1"]
            #[doc = "line2"]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(base, &attrs).unwrap();
        let lines: Vec<_> = result.lines().collect();
        // We only expect line1 and line2 once each
        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&"/// line1"));
        assert!(lines.contains(&"/// line2"));
    }

    /// 6) If neither the base docs nor the attrs actually yield any doc lines, we get None.
    #[test]
    fn test_no_doc_lines_in_attrs() {
        let base = None;
        let attrs = Some(
            r#"
            #[allow(something)]
            #[cfg(feature = "whatever")]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(base, &attrs);
        assert!(result.is_none(), "Expected None because no #[doc=...] lines present");
    }

    /// 7) If the base docs contain different lines in arbitrary order, plus doc lines from attrs, they get sorted in ascending order.
    #[test]
    fn test_btreeset_sorts_merged_lines() {
        let base = Some("/// z-line\n/// a-line\n/// middle-line".to_string());
        let attrs = Some(
            r#"
            #[doc = "b-line"]
            #[doc = "y-line"]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(base, &attrs).unwrap();
        // BTreeSet default is alphabetical. So lines: "/// a-line", "/// b-line", "/// middle-line", "/// y-line", "/// z-line"
        let expected = r#"/// a-line
/// b-line
/// middle-line
/// y-line
/// z-line"#;
        assert_eq!(
            result, expected,
            "Should produce lines in alphabetical order (due to BTreeSet usage)"
        );
    }

    /// 8) If there's leading/trailing spaces in the doc = "...", we trim them.
    #[test]
    fn test_trimming_doc_string_in_attr() {
        let base = None;
        let attrs = Some(
            r#"
            #[doc   =   "   trimmed   "]
        "#
            .to_string(),
        );
        let result = merge_doc_attrs(base, &attrs).unwrap();
        // We expect "/// trimmed"
        let expected = "/// trimmed";
        assert_eq!(result, expected);
    }

    /// 9) If the attribute has a doc but no quotes or the pattern doesn't match, it's ignored.
    #[test]
    fn test_non_matching_doc_attribute_ignored() {
        let base = None;
        let attrs = Some(r#"#[doc = not_a_string]"#.to_string());
        let result = merge_doc_attrs(base, &attrs);
        assert!(result.is_none(), "No valid doc lines matched => None");
    }
}
