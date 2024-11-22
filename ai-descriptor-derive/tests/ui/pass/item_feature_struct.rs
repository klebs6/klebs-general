use ai_descriptor_derive::*;
use ai_descriptor_trait::*;

#[derive(ItemFeature, Hash, Debug, Clone, PartialEq, Eq)]
#[ai("Follows a custom rhyme scheme: {0}.")]
pub struct CustomRhymeScheme(String);

fn main() {}
