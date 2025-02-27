// ---------------- [ File: src/existing_x_macro.rs ]
crate::ix!();

#[derive(Builder,Getters,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct ExistingXMacro {
    text:  String,     // e.g. "x!{command_runner}"
    range: TextRange,  // so we can remove it from the file

    /// Any immediately preceding line comments that should stay with this macro
    /// (e.g. doc comments or inline `// comment` lines).
    /// We collect them so that, when we move the macro to the top block, we can
    /// keep those comments attached to the macro in the new location.
    #[builder(default = "String::new()")]
    leading_comments: String,
}


// A small struct to represent any macro we plan to unify at the top block,
// whether it came from the old file or from the new_top_block text.
#[derive(Clone,Builder,Getters,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct TopBlockMacro {
    stem: String,
    leading_comments: String, // e.g. `// top block\n`
}
