pub use ai_descriptor_derive::*;
pub use ai_descriptor_trait::*;
pub use plural_derive::*;
pub use plural_trait::*;

pub trait OpenConversation {
    fn open_conversation(&self) -> &'static str;
}
