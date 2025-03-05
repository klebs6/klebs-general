// ---------------- [ File: workspacer-consolidate/src/consolidated_item.rs ]
crate::ix!();

/// An enum representing any consolidated item that can live in a module.
/// You could also store them separately if you like, but an enum is convenient.
#[derive(Debug)]
pub enum ConsolidatedItem {
    Fn(CrateInterfaceItem<ast::Fn>),
    Struct(CrateInterfaceItem<ast::Struct>),
    Enum(CrateInterfaceItem<ast::Enum>),
    Trait(CrateInterfaceItem<ast::Trait>),
    TypeAlias(CrateInterfaceItem<ast::TypeAlias>),
    Macro(CrateInterfaceItem<ast::MacroRules>),
    Module(ModuleInterface),
    ImplBlock(ImplBlockInterface),

    // --- a special "test" variant ---
    #[cfg(test)]
    MockTest(String),
}

impl fmt::Display for ConsolidatedItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsolidatedItem::Fn(item)        => write!(f, "{}", item),
            ConsolidatedItem::Struct(item)    => write!(f, "{}", item),
            ConsolidatedItem::Enum(item)      => write!(f, "{}", item),
            ConsolidatedItem::Trait(item)     => write!(f, "{}", item),
            ConsolidatedItem::TypeAlias(item) => write!(f, "{}", item),
            ConsolidatedItem::Macro(item)     => write!(f, "{}", item),
            ConsolidatedItem::Module(mi)      => write!(f, "{}", mi),
            ConsolidatedItem::ImplBlock(ib)   => write!(f, "{}", ib),

            #[cfg(test)]
            ConsolidatedItem::MockTest(ib)   => write!(f, "{}", ib),
        }
    }
}
