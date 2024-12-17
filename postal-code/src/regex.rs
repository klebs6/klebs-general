crate::ix!();

#[macro_export]
macro_rules! define_postal_code_regexes {
    ($id:ident, { $($c:ident => $pattern:expr),+ $(,)? }) => {
        paste::paste! {
            // Define a unique module name based on the user-supplied $id
            #[allow(non_snake_case)]
            mod [<__internal_ $id >] {
                use super::*;
                use once_cell::sync::Lazy;
                use ::regex::Regex; // Use ::regex to disambiguate
                
                $(
                    #[allow(non_upper_case_globals)]
                    pub static [<REGEX_ $c>]: Lazy<Result<Regex, PostalCodeConstructionError>> = Lazy::new(|| {
                        Regex::new($pattern)
                            .map_err(|_| PostalCodeConstructionError::RegexInitializationError { country: Country::$c })
                    });
                )+
            }

            $(
                pub use [<__internal_ $id >]::[<REGEX_ $c>];
            )+
        }
    }
}

define_postal_code_regexes!{
    Set1, {
        USA           => r"^\d{5}(-\d{4})?$",
        Canada        => r"^[A-Za-z]\d[A-Za-z] ?\d[A-Za-z]\d$",
        UnitedKingdom => r"^(GIR 0AA)$|((([A-Z]{1,2}[0-9][A-Z0-9]?)|(([A-Z]{1,2}[0-9]{1,2})|(([A-Z]{1,2}[0-9][A-Z])|([A-Z]{1,2}[0-9]{2})))) [0-9][A-Z]{2})$",
        France        => r"^\d{5}$",
    }
}

// NOTE: these two `define_postal_code_regexes` invoations are only separated to show an example of
// how a user can install more regexes using this macro
//
define_postal_code_regexes!{
    Set2, {
        Germany => r"^\d{5}$",
        Italy   => r"^\d{5}$",
    }
}
