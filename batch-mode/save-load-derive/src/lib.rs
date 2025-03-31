// ---------------- [ File: save-load-derive/src/lib.rs ]
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput,parse_macro_input};
use tracing::*;

#[proc_macro_derive(SaveLoad)]
pub fn save_load_derive(input: TokenStream) -> TokenStream {
    trace!("Entered save_load_derive procedural macro.");

    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;

    debug!(
        "Generating `SaveToFile` and `LoadFromFile` impl for `{}` using standard trait bounds.",
        ident
    );

    // We generate code with T: Serialize + for<'de> Deserialize<'de> trait bounds.
    // If the user hasn't derived or otherwise implemented these, the compiler
    // will produce a standard error message about missing trait bounds instead.
    //
    // We also reference the traits and error type from `save_load_traits` using
    // fully-qualified paths. Adjust them as necessary for your actual crate setup.
    let expanded = quote! {

        #[async_trait]
        impl SaveToFile for #ident
        where
            #ident: ::serde::Serialize,
        {
            type Error = SaveLoadError;

            async fn save_to_file(
                &self,
                filename: impl AsRef<std::path::Path> + Send
            ) -> Result<(), Self::Error> {
                tracing::debug!(
                    "Attempting to save `{}` to file: {:?}",
                    stringify!(#ident),
                    filename.as_ref()
                );

                let serialized = serde_json::to_string_pretty(self)?;

                tokio::fs::write(filename.as_ref(), &serialized).await?;

                tracing::info!(
                    "Successfully saved `{}` to file: {:?}",
                    stringify!(#ident),
                    filename.as_ref()
                );
                Ok(())
            }
        }

        #[async_trait]
        impl LoadFromFile for #ident
        where
            #ident: for<'de> ::serde::de::Deserialize<'de>,
        {
            type Error = SaveLoadError;

            async fn load_from_file(
                filename: impl AsRef<std::path::Path> + Send
            ) -> Result<Self, Self::Error> {
                tracing::debug!(
                    "Attempting to load `{}` from file: {:?}",
                    stringify!(#ident),
                    filename.as_ref()
                );

                let content = tokio::fs::read_to_string(filename.as_ref()).await?;

                Ok(serde_json::from_str(&content)?)
            }
        }
    };

    trace!(
        "Completed macro expansions for `{}`. Returning generated code.",
        ident
    );
    expanded.into()
}
