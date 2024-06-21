use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn disable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Return an empty TokenStream to completely remove the annotated item
    TokenStream::new()
}
