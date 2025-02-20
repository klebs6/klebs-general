// ---------------- [ File: src/module_interface.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
// Representation of a mod block
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct ModuleInterface {
    docs:    Option<String>,
    attrs:   Option<String>,
    mod_name: String,
    items:   Vec<ConsolidatedItem>,
}

impl ModuleInterface {
    pub fn new(docs: Option<String>, attrs: Option<String>, mod_name: String) -> Self {
        Self { docs, attrs, mod_name, items: vec![] }
    }
    pub fn add_item(&mut self, item: ConsolidatedItem) {
        self.items.push(item);
    }
}

impl fmt::Display for ModuleInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.items.is_empty() {
            return Ok(());
        }
        if let Some(attrs) = &self.attrs {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        if let Some(doc_text) = &self.docs {
            writeln!(f, "{}", doc_text)?;
        }
        writeln!(f, "mod {} {{", self.mod_name)?;
        for (i, item) in self.items.iter().enumerate() {
            let display_str = format!("{}", item);
            for line in display_str.lines() {
                writeln!(f, "    {}", line)?;
            }
            if i + 1 < self.items.len() {
                writeln!(f)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
