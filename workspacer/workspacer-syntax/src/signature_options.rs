crate::ix!();

/// Control how we generate a signature string.
/// - `fully_expand` => whether to expand all fields/variants or to keep placeholders.
/// - `include_docs` => whether to prepend doc comments to the signature text.
#[derive(Builder,Getters,Setters,Debug, Clone, Copy)]
#[builder(setter(into))]
#[getset(get="pub",set="pub")]
pub struct SignatureOptions {
    fully_expand:  bool,
    include_docs:  bool,

    #[builder(default=false)]
    add_semicolon: bool,
}

impl Default for SignatureOptions {
    fn default() -> Self {
        SignatureOptions {
            fully_expand:  true,  // By default, show everything
            include_docs:  true,  // By default, show doc lines
            add_semicolon: false,
        }
    }
}
