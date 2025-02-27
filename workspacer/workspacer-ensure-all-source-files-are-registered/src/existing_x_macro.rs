// ---------------- [ File: src/existing_x_macro.rs ]
crate::ix!();

#[derive(Builder,Getters,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct ExistingXMacro {
    text:  String,     // e.g. "x!{command_runner}"
    range: TextRange, // so we can remove it from the file
}
