crate::ix!();

/// Enum to specify parsing strategy.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum JsonParsingStrategy {
    WithoutRepair,
    WithRepair,
}

