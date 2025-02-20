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
