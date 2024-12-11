crate::ix!();

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Middle {
    Name(MiddleName),
    Initial(MiddleInitial),
}

impl From<char> for Middle {
    fn from(x: char) -> Middle {
        Middle::Initial(MiddleInitial::from(x))
    }
}

impl From<&str> for Middle {
    fn from(x: &str) -> Middle {
        Middle::Name(MiddleName::from(x))
    }
}


impl Middle {
    /// Convert the middle component to a human-readable string.
    pub fn to_string(&self) -> String {
        match self {
            Middle::Name(n)    => n.to_string(),
            Middle::Initial(i) => i.to_string(),
        }
    }
}

#[macro_export]
macro_rules! middle {
    ($c:literal) => {
        Middle::Initial(MiddleInitial::from($c))
    };
    ($str:expr) => {
        Middle::Name(MiddleName::from($str))
    };
}
