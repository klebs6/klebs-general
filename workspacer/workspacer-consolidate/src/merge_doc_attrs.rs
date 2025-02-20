// ---------------- [ File: src/merge_doc_attrs.rs ]
crate::ix!();

/// Merge any `#[doc="..."]` attribute lines into the doc string
/// so we don’t double-print them. 
pub fn merge_doc_attrs(
    base_docs: Option<String>,
    maybe_attrs: &Option<String>
) -> Option<String> {
    let mut doc_set = BTreeSet::new();
    if let Some(ref base) = base_docs {
        for line in base.lines() {
            doc_set.insert(line.to_string());
        }
    }

    if let Some(attr_text) = maybe_attrs {
        // e.g. #[doc = "some line"]
        let re = Regex::new(r#"#\[doc\s*=\s*"([^"]*)"\s*\]"#).unwrap();
        for line in attr_text.lines() {
            if let Some(caps) = re.captures(line.trim()) {
                let doc_str = caps[1].trim();
                let doc_line = format!("/// {}", doc_str);
                doc_set.insert(doc_line);
            }
        }
    }

    if doc_set.is_empty() {
        None
    } else {
        Some(doc_set.into_iter().collect::<Vec<_>>().join("\n"))
    }
}
