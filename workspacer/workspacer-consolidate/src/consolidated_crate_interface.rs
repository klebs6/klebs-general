// ---------------- [ File: workspacer-consolidate/src/consolidated_crate_interface.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Clone,MutGetters,Getters,Debug)]
#[getset(get="pub",get_mut="pub")]
pub struct ConsolidatedCrateInterface {
    fns:          Vec<CrateInterfaceItem<ast::Fn>>,
    structs:      Vec<CrateInterfaceItem<ast::Struct>>,
    enums:        Vec<CrateInterfaceItem<ast::Enum>>,
    traits:       Vec<CrateInterfaceItem<ast::Trait>>,
    type_aliases: Vec<CrateInterfaceItem<ast::TypeAlias>>,
    macros:       Vec<CrateInterfaceItem<ast::MacroRules>>,
    impls:        Vec<ImplBlockInterface>,
    modules:      Vec<ModuleInterface>,
}

unsafe impl Send for ConsolidatedCrateInterface {}
unsafe impl Sync for ConsolidatedCrateInterface {}

impl fmt::Display for ConsolidatedCrateInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! print_items {
            ($vec:expr, $f:expr) => {
                for (i, item) in $vec.iter().enumerate() {
                    writeln!($f, "{}", item)?;
                    if i + 1 < $vec.len() {
                        writeln!($f)?;
                    }
                }
            };
        }

        print_items!(self.enums, f);
        print_items!(self.traits, f);
        print_items!(self.type_aliases, f);
        print_items!(self.macros, f);
        print_items!(self.structs, f);
        print_items!(self.fns, f);
        print_items!(self.impls, f);
        print_items!(self.modules, f);

        Ok(())
    }
}

impl ConsolidatedCrateInterface {
    pub fn new() -> Self {
        Self {
            fns:          vec![],
            structs:      vec![],
            enums:        vec![],
            traits:       vec![],
            type_aliases: vec![],
            macros:       vec![],
            impls:        vec![],
            modules:      vec![],
        }
    }

    pub fn add_fn(&mut self, item: CrateInterfaceItem<ast::Fn>) { self.fns.push(item); }
    pub fn add_struct(&mut self, item: CrateInterfaceItem<ast::Struct>) { self.structs.push(item); }
    pub fn add_enum(&mut self, item: CrateInterfaceItem<ast::Enum>) { self.enums.push(item); }
    pub fn add_trait(&mut self, item: CrateInterfaceItem<ast::Trait>) { self.traits.push(item); }
    pub fn add_type_alias(&mut self, item: CrateInterfaceItem<ast::TypeAlias>) { self.type_aliases.push(item); }
    pub fn add_macro(&mut self, item: CrateInterfaceItem<ast::MacroRules>) { self.macros.push(item); }
    pub fn add_impl(&mut self, ib: ImplBlockInterface) { self.impls.push(ib); }
    pub fn add_module(&mut self, ib: ModuleInterface) { self.modules.push(ib); }
}
