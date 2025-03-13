crate::ix!();

pub trait RehydrateFromSignature: GenerateSignature + Sized {
    /// Attempt to reconstruct `Self` from a signature string (presumably
    /// produced by `generate_signature()`).
    ///
    /// Returns `None` if reconstruction fails.
    fn rehydrate_from_signature(signature_source: &str) -> Option<Self>;
}

