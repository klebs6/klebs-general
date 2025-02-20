// ---------------- [ File: src/impl_block_interface.rs ]
crate::ix!();

#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct ImplBlockInterface {
    docs:           Option<String>,
    attributes:     Option<String>,
    signature_text: String,
    methods:        Vec<crate::crate_interface_item::CrateInterfaceItem<ast::Fn>>,
    type_aliases:   Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>>,
}

impl ImplBlockInterface {
    pub fn new(
        docs:           Option<String>,
        attributes:     Option<String>,
        signature_text: String,
        methods:        Vec<CrateInterfaceItem<ast::Fn>>,
        type_aliases:   Vec<CrateInterfaceItem<ast::TypeAlias>>,
    ) -> Self {
        Self {
            docs,
            attributes,
            signature_text,
            methods,
            type_aliases,
        }
    }
}

impl fmt::Display for ImplBlockInterface {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // If we choose to always show the impl block, even if empty,
        // do something like:

        if let Some(d) = &self.docs {
            for line in d.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        if let Some(a) = &self.attributes {
            for line in a.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // If no items, show single line “impl Something for T {}”
        if self.methods.is_empty() && self.type_aliases.is_empty() {
            return write!(f, "{} {{}}", self.signature_text);
        }

        // Otherwise, multi-line display
        writeln!(f, "{} {{", self.signature_text)?;

        for ta in &self.type_aliases {
            let text = format!("{}", ta);
            for line in text.lines() {
                writeln!(f, "    {}", line)?;
            }
        }
        for m in &self.methods {
            let text = format!("{}", m);
            for line in text.lines() {
                writeln!(f, "    {}", line)?;
            }
        }

        writeln!(f, "}}")
    }
}
