crate::ix!();

pub trait Validate {

    fn validate(&self) -> bool;
}
