crate::ix!();

/// Compile‑time fragments required to *generate* a value for an individual
/// field across all constructor modes.
///
/// All fields are private; read‑only access is provided via `getset`.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct FieldGenerationTokens {
    random:      TokenStream2,
    uniform:     TokenStream2,
    random_env:  TokenStream2,
    uniform_env: TokenStream2,
    provider_types: Vec<Type>,
    rand_bound: TokenStream2,
}


