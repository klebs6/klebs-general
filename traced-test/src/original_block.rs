crate::ix!();

pub trait HasOriginalBlock {

    fn original_block(&self) -> &syn::Block;
}
