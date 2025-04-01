crate::ix!();

#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct ReadmeWriterConfig {
    skip_docs:             bool,
    skip_fn_bodies:        bool,
    include_test_items:    bool,
    include_private_items: bool,
    max_interface_length:  Option<usize>,
}

impl Default for ReadmeWriterConfig {

    fn default() -> Self {
        Self {
            skip_docs:             false,
            skip_fn_bodies:        false,
            include_test_items:    false,
            include_private_items: false,
            max_interface_length:  Some(4092),
        }
    }
}
