crate::ix!();

pub trait IsValidVersion {

    fn is_valid_version(version: &str) -> bool;
}
