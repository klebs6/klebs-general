// ---------------- [ File: src/should_trace.rs ]
crate::ix!();

pub trait ShouldTrace {
    fn should_trace_on_success(&self) -> bool;
    fn should_trace_on_failure(&self) -> bool;
}

impl TracedTestGenerator {

    pub fn define_should_trace_trait(&self) -> TokenStream2 {
        quote!{
            pub trait ShouldTrace {
                fn should_trace_on_success(&self) -> bool;
                fn should_trace_on_failure(&self) -> bool;
            }
        }
    }
}
