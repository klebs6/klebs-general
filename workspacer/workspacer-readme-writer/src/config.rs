crate::ix!();

#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct ReadmeWriterConfig {
    #[builder(default = "false")]      skip_docs:             bool,
    #[builder(default = "false")]      skip_fn_bodies:        bool,
    #[builder(default = "false")]      include_test_items:    bool,
    #[builder(default = "false")]      include_private_items: bool,
    #[builder(default = "Some(4096)")] max_interface_length:  Option<usize>,
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
