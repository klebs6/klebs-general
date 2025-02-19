// ---------------- [ File: workspacer-crate/src/crate_interface_item.rs ]
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

    /// Create a new interface item. We unify doc lines from:
    /// - the `docs` param (which typically comes from `extract_docs(...)`),
    /// - and any lines from `attributes` that are `#[doc = "..."]`,
    ///   if you want to unify them as well. Then we remove those from `attributes`.
    pub fn new(
        item: T,
        docs: Option<String>,
        attributes: Option<String>,
        body_source: Option<String>,
    ) -> Self {
        // 1) Gather doc lines from `docs` plus doc lines from attributes
        let mut final_docs = merge_doc_attrs(docs.clone(), &attributes);

        // 2) Filter out the doc attributes from `attributes` 
        //    so we don't print them again. 
        let filtered_attrs = attributes.map(|txt| {
            txt.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    // skip if it's #![doc(...)] or #[doc(...)] 
                    !(trimmed.starts_with("#[doc =") || trimmed.starts_with("#![doc ="))
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
        let signature = self.item.generate_signature(self.docs.as_ref());
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

/// Merge any doc lines from the `#[doc = "..."]` attributes into the `docs` string,
/// deduplicating them. For simplicity, we treat them all as if they start with `///`.
fn merge_doc_attrs(base_docs: Option<String>, maybe_attrs: &Option<String>) -> Option<String> {
    use regex::Regex;

    // gather any doc lines from base_docs into a set
    let mut doc_set = std::collections::BTreeSet::new();
    if let Some(ref base) = base_docs {
        for line in base.lines() {
            doc_set.insert(line.to_string());
        }
    }

    // parse out lines from attributes that look like: #[doc = "..."]
    if let Some(attr_text) = &maybe_attrs {
        // e.g. line: #[doc = "Some doc line"]
        let re = Regex::new(r#"#\[doc\s*=\s*"([^"]*)"\s*\]"#).unwrap();

        for line in attr_text.lines() {
            if let Some(caps) = re.captures(line.trim()) {
                // The doc string is caps[1]
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

/// Returns how many leading space characters are at the start of `line`.
fn leading_spaces(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}
