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
    #[builder(default = "None")]
    leading_comments: Option<String>,
}


// A small struct to represent any macro we plan to unify at the top block,
// whether it came from the old file or from the new_top_block text.
#[derive(Clone,Builder,Getters,Debug)]
#[builder(setter(into),build_fn(name = "private_build"))]
#[getset(get="pub")]
pub struct TopBlockMacro {
    stem: String,

    #[builder(default = "None")]
    leading_comments: Option<String>, // e.g. `// top block\n`
}

impl TopBlockMacroBuilder {

    pub fn build(&self) -> Result<TopBlockMacro, &'static str> {
        // We must have a valid stem
        let stem_str = self.stem.clone().ok_or("stem is required")?;

        // Convert any leading_comments to a trimmed string
        // then if it’s empty => store None
        let leading_comments = match self.leading_comments.clone()
        {
            None          => None,
            Some(None)    => None,
            Some(Some(c)) => {
                let trimmed = c.trim().to_string();
                match trimmed.is_empty() {
                    true  => None,
                    false => Some(trimmed),
                }
            },
        };

        Ok(TopBlockMacro {
            stem: stem_str,
            leading_comments,
        })
    }
}
