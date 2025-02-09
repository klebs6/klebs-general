// ---------------- [ File: workspacer-consolidate/src/consolidated_crate_interface.rs ]
// ---------------- [ File: workspacer-consolidate/src/consolidated_crate_interface.rs ]
crate::ix!();

pub struct ConsolidatedCrateInterface {
    traits:  Vec<CrateInterfaceItem<ast::Trait>>,
    fns:     Vec<CrateInterfaceItem<ast::Fn>>,
    structs: Vec<CrateInterfaceItem<ast::Struct>>,
    enums:   Vec<CrateInterfaceItem<ast::Enum>>,
    types:   Vec<CrateInterfaceItem<ast::TypeAlias>>,
    macros:  Vec<CrateInterfaceItem<ast::MacroRules>>,
}

unsafe impl Send for ConsolidatedCrateInterface {}

impl fmt::Display for ConsolidatedCrateInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.get_traits() {
            writeln!(f, "{}", item)?;
        }

        for item in self.get_fns() {
            writeln!(f, "{}", item)?;
        }

        for item in self.get_structs() {
            writeln!(f, "{}", item)?;
        }

        for item in self.get_enums() {
            writeln!(f, "{}", item)?;
        }

        for item in self.get_types() {
            writeln!(f, "{}", item)?;
        }

        for item in self.get_macros() {
            writeln!(f, "{}", item)?;
        }

        Ok(())
    }
}

impl ConsolidatedCrateInterface {

    // Constructor for creating an empty interface
    pub fn new() -> Self {
        Self {
            traits:  Vec::new(),
            fns:     Vec::new(),
            structs: Vec::new(),
            enums:   Vec::new(),
            types:   Vec::new(),
            macros:  Vec::new(),
        }
    }

    // Methods to get traits, functions, structs, enums, etc.
    pub fn get_traits(&self) -> &Vec<CrateInterfaceItem<ast::Trait>> {
        &self.traits
    }

    pub fn get_fns(&self) -> &Vec<CrateInterfaceItem<ast::Fn>> {
        &self.fns
    }

    pub fn get_structs(&self) -> &Vec<CrateInterfaceItem<ast::Struct>> {
        &self.structs
    }

    pub fn get_enums(&self) -> &Vec<CrateInterfaceItem<ast::Enum>> {
        &self.enums
    }

    pub fn get_types(&self) -> &Vec<CrateInterfaceItem<ast::TypeAlias>> {
        &self.types
    }

    pub fn get_macros(&self) -> &Vec<CrateInterfaceItem<ast::MacroRules>> {
        &self.macros
    }

    // Methods to add new items
    pub fn add_trait(&mut self, item: CrateInterfaceItem<ast::Trait>) {
        self.traits.push(item);
    }

    pub fn add_fn(&mut self, item: CrateInterfaceItem<ast::Fn>) {
        self.fns.push(item);
    }

    pub fn add_struct(&mut self, item: CrateInterfaceItem<ast::Struct>) {
        self.structs.push(item);
    }

    pub fn add_enum(&mut self, item: CrateInterfaceItem<ast::Enum>) {
        self.enums.push(item);
    }

    pub fn add_type(&mut self, item: CrateInterfaceItem<ast::TypeAlias>) {
        self.types.push(item);
    }

    pub fn add_macro(&mut self, item: CrateInterfaceItem<ast::MacroRules>) {
        self.macros.push(item);
    }
}
