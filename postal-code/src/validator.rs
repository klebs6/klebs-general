crate::ix!();

/// Trait that all country-specific postal code validators must implement.
pub trait PostalCodeValidator {
    fn validate(&self, code: &str) -> bool;
}

/// Macro to define validators easily:
#[macro_export] macro_rules! generate_postal_code_validator {
    ($name:ident, $country:expr, $regex:ident) => {
        pub struct $name;

        impl PostalCodeValidator for $name {
            fn validate(&self, code: &str) -> bool {
                $regex.as_ref().map_or(false, |r| r.is_match(code))
            }
        }
    };
}

generate_postal_code_validator!{
    UsaValidator, 
    Country::USA, 
    REGEX_USA
}

generate_postal_code_validator!{
    CanadaValidator, 
    Country::Canada, 
    REGEX_Canada
}

generate_postal_code_validator!{
    UkValidator, 
    Country::UnitedKingdom, 
    REGEX_UnitedKingdom
}

generate_postal_code_validator!{
    FranceValidator, 
    Country::France, 
    REGEX_France
}

generate_postal_code_validator!{
    GermanyValidator, 
    Country::Germany, 
    REGEX_Germany
}

generate_postal_code_validator!{
    ItalyValidator, 
    Country::Italy, 
    REGEX_Italy
}

pub trait GetPostalCodeValidator {

    fn get_postal_code_validator(&self) 
        -> Option<Box<dyn PostalCodeValidator + Send + Sync>>;
}

impl GetPostalCodeValidator for Country {

    /// Return a validator for the given country if supported.
    fn get_postal_code_validator(&self) 
        -> Option<Box<dyn PostalCodeValidator + Send + Sync>> 
    {
        match self {
            Country::USA           => Some(Box::new(UsaValidator)),
            Country::Canada        => Some(Box::new(CanadaValidator)),
            Country::UnitedKingdom => Some(Box::new(UkValidator)),
            Country::France        => Some(Box::new(FranceValidator)),
            Country::Germany       => Some(Box::new(GermanyValidator)),
            Country::Italy         => Some(Box::new(ItalyValidator)),
            _                      => None, // Unsupported countries return None
        }
    }
}
