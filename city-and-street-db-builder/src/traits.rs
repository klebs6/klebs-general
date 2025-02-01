// ---------------- [ File: src/traits.rs ]
crate::ix!();

pub trait ValidateWith {

    type Validator;
    type Error;

    fn validate_with(
        &self, 
        validator: &Self::Validator,
    ) -> Result<(),Self::Error>;
}

pub trait Mock {

    fn mock() -> Self;
}

pub trait MockI {

    fn mock(i:usize) -> Self;
}

pub trait MockForRegion {

    fn mock_for_region(region: &WorldRegion) -> Self;
}
