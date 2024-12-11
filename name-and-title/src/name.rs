crate::ix!();

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FirstName(String);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LastName(String);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct MiddleName(String);

//----------------------------------------
macro_rules! impl_name {
    ($ty:ident) => {
        impl From<&str> for $ty  { fn from(x: &str) -> $ty  { $ty(x.to_string()) } }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    }
}

impl_name!{FirstName}
impl_name!{LastName}
impl_name!{MiddleName}

#[macro_export]
macro_rules! first {
    ($str:expr) => {
        FirstName::from($str)
    }
}

#[macro_export]
macro_rules! last {
    ($str:expr) => {
        LastName::from($str)
    }
}
