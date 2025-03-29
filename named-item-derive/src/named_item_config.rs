// ---------------- [ File: src/named_item_config.rs ]
crate::ix!();

/// Configuration extracted from `#[named_item(...)]`.
#[derive(Debug,Clone,Builder,Getters)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct NamedItemConfig {
    /// Optional default name if `default_name="foo"`.
    default_name: Option<String>,
    /// If `aliases="true"`, the struct must have `aliases: Vec<String>`.
    aliases: bool,
    /// If `default_aliases="foo,bar"`, we store them here.
    default_aliases: Vec<String>,
    /// If `history="true"`, the struct must have `name_history: Vec<String>`.
    history: bool,
}
