// ---------------- [ File: src/original.rs ]
crate::ix!();

pub trait HasOriginalBlock {

    fn original_block(&self) -> &syn::Block;
}

pub trait HasOriginalItem {

    type Item;

    fn original(&self) -> Self::Item;
}
