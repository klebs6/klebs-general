use ai_descriptor_derive::*;
use ai_descriptor_trait::*;
use std::borrow::Cow;

#[derive(ItemWithFeatures)]
#[ai("Potion")]
struct Potion {
    #[ai(feature_if_none = "No side effects.")]
    side_effects: Option<ExampleStruct>,
}

struct ExampleStruct;
impl ItemFeature for ExampleStruct {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed("just an example")
    }
}

fn main() {}

