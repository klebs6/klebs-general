// ---------------- [ File: src/validate_parse_result.rs ]
crate::ix!();

/// Which logical field is currently being edited by the user?
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ValidateCursorField {
    Zip,
    City,
    HouseNumber,
    Street,
}

/// The partial parse of the user's `validate` command line.
/// For example, if the user typed:
///   `validate 20016 washington new mex`
/// then:
///   - zip_part = "20016"
///   - city_parts = ["washington"]
///   - house_number_part = None
///   - street_parts = ["new", "mex"] (the user is still typing "mex")
///   - cursor_field = Street
#[derive(Builder,Getters,Setters)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct ValidateParseResult {
    zip_part:            String,
    city_parts:          Vec<String>,
    house_number_part:   Option<String>,
    street_parts:        Vec<String>,
    cursor_field:        ValidateCursorField,
}
