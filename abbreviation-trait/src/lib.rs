pub trait Abbreviation {
    fn abbreviation(&self) -> &'static str;
}

/// A trait for parsing an entity (typically) from a country code abbreviation.
/// This can be ISO 3166 Alpha-2 (e.g. "US") or Alpha-3 (e.g. "USA"), or any
/// other recognized short code as needed.
pub trait TryFromAbbreviation: Sized {
    /// The associated error type returned upon failure.
    type Error;

    /// Attempt to convert an abbreviation into `Self`.
    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub enum TryFromAbbreviationError {
    /// The given abbreviation is not recognized in this region.
    InvalidAbbreviation,
}
