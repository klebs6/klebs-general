// ---------------- [ File: workspacer-consolidate/src/consolidated_item.rs ]
crate::ix!();

/// An enum representing any consolidated item that can live in a module.
/// You could also store them separately if you like, but an enum is convenient.
#[derive(Serialize,Deserialize,Clone,Debug)]
pub enum ConsolidatedItem {
    Fn(CrateInterfaceItem<ast::Fn>),
    Struct(CrateInterfaceItem<ast::Struct>),
    Enum(CrateInterfaceItem<ast::Enum>),
    Trait(CrateInterfaceItem<ast::Trait>),
    TypeAlias(CrateInterfaceItem<ast::TypeAlias>),
    Macro(CrateInterfaceItem<ast::MacroRules>),
    MacroCall(CrateInterfaceItem<ast::MacroCall>),
    Module(ModuleInterface),
    ImplBlock(ImplBlockInterface),

    // --- a special "test" variant ---
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
            ConsolidatedItem::MacroCall(item)  => write!(f, "{}", item),
            ConsolidatedItem::Module(mi)      => write!(f, "{}", mi),
            ConsolidatedItem::ImplBlock(ib)   => write!(f, "{}", ib),

            ConsolidatedItem::MockTest(ib)   => write!(f, "{}", ib),
        }
    }
}

impl ConsolidatedItem {

    /// Helper to get the start offset from a `ConsolidatedItem`.
    pub fn item_start(&self) -> TextSize {
        match self {
            ConsolidatedItem::Fn(f)         => f.text_range().start(),
            ConsolidatedItem::Struct(s)     => s.text_range().start(),
            ConsolidatedItem::Enum(e)       => e.text_range().start(),
            ConsolidatedItem::Trait(t)      => t.text_range().start(),
            ConsolidatedItem::TypeAlias(ta) => ta.text_range().start(),
            ConsolidatedItem::Macro(m)      => m.text_range().start(),
            ConsolidatedItem::MacroCall(m)  => m.text_range().start(),
            ConsolidatedItem::ImplBlock(i)  => i.text_range().start(),
            ConsolidatedItem::Module(mo)    => mo.text_range().start(),
            ConsolidatedItem::MockTest(_)   => TextSize::from(0),
        }
    }
}
