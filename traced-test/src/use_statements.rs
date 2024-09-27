crate::ix!();

impl TracedTestGenerator {

    pub fn use_statements_for_new_block(&self) -> TokenStream2 {

        let mut use_statements = TokenStream2::new();

        use_statements.extend(quote!{ use tracing::*; });
        use_statements.extend(quote!{ use std::panic::AssertUnwindSafe; });

        if self.is_async() {
            use_statements.extend(quote!{ use tokio::task; });
        }

        use_statements
    }
}

