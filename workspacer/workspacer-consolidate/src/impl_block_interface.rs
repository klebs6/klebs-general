crate::ix!();

#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct ImplBlockInterface {
    docs:           Option<String>,
    attributes:     Option<String>,
    signature_text: String,
    methods:        Vec<CrateInterfaceItem<ast::Fn>>,
    type_aliases:   Vec<CrateInterfaceItem<ast::TypeAlias>>,
}

impl ImplBlockInterface {
    pub fn new(
        docs: Option<String>,
        attributes: Option<String>,
        signature_text: String,
        methods: Vec<CrateInterfaceItem<ast::Fn>>,
        type_aliases: Vec<CrateInterfaceItem<ast::TypeAlias>>,
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
        // doc lines first
        if let Some(d) = &self.docs {
            for line in d.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        // then attributes
        if let Some(a) = &self.attributes {
            let lines: Vec<_> = a.lines()
                .filter(|l| !l.trim().starts_with("#[doc =")) // skip doc= if present
                .collect();
            for line in lines {
                writeln!(f, "{}", line)?;
            }
        }

        writeln!(f, "{} {{", self.signature_text)?;

        // Indent each type alias and method the same
        for ta in &self.type_aliases {
            for l in format!("{}", ta).lines() {
                writeln!(f, "    {}", l)?;
            }
        }
        for m in &self.methods {
            for l in format!("{}", m).lines() {
                writeln!(f, "    {}", l)?;
            }
        }

        writeln!(f, "}}")?;
        Ok(())
    }
}
