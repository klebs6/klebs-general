// ---------------- [ File: save-load-traits/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{save_load_traits}
x!{load_from_directory}
x!{errors}
x!{impl_default_macro}
x!{impl_for_vec}
x!{impl_for_string}
x!{impl_for_hashmap}

pub use async_trait::async_trait;
pub use tokio;
