crate::ix!();

/// Represents the title for a person.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum PersonTitle {
    Mr,
    Mrs,
    Miss,
    Dr,
    Prof,
    Eng,
    Capt,
    Mx,
    Rev,
    Other(String),
}

impl fmt::Display for PersonTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonTitle::Mr       => write!(f, "Mr."),
            PersonTitle::Mrs      => write!(f, "Mrs."),
            PersonTitle::Miss     => write!(f, "Miss"),
            PersonTitle::Dr       => write!(f, "Dr."),
            PersonTitle::Prof     => write!(f, "Prof."),
            PersonTitle::Eng      => write!(f, "Eng."),
            PersonTitle::Capt     => write!(f, "Capt."),
            PersonTitle::Mx       => write!(f, "Mx."),
            PersonTitle::Rev      => write!(f, "Rev."),
            PersonTitle::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for PersonTitle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mr."   => Ok(PersonTitle::Mr),
            "Mrs."  => Ok(PersonTitle::Mrs),
            "Miss"  => Ok(PersonTitle::Miss),
            "Dr."   => Ok(PersonTitle::Dr),
            "Prof." => Ok(PersonTitle::Prof),
            "Eng."  => Ok(PersonTitle::Eng),
            "Capt." => Ok(PersonTitle::Capt),
            "Mx."   => Ok(PersonTitle::Mx),
            "Rev."  => Ok(PersonTitle::Rev),
            other   => Ok(PersonTitle::Other(other.to_string())),
        }
    }
}

#[cfg(test)]
mod person_title_tests {
    use super::*;

    #[test]
    fn test_other_title() {
        let title = PersonTitle::from_str("Sir").unwrap();
        assert_eq!(title, PersonTitle::Other("Sir".to_string()));
    }

    #[test]
    fn test_known_title_parse() {
        let title = PersonTitle::from_str("Dr.").unwrap();
        assert_eq!(title, PersonTitle::Dr);
    }
}
