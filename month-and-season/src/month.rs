crate::ix!();

#[derive(EnumIter,Hash,Copy,Debug,PartialEq,Eq,Clone,Serialize,Deserialize)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December
}

impl Month {
    /// Convert a month number (1 to 12) into a `Month`.
    pub fn from_number(n: u8) -> Result<Self, &'static str> {
        match n {
            1 => Ok(Month::January),
            2 => Ok(Month::February),
            3 => Ok(Month::March),
            4 => Ok(Month::April),
            5 => Ok(Month::May),
            6 => Ok(Month::June),
            7 => Ok(Month::July),
            8 => Ok(Month::August),
            9 => Ok(Month::September),
            10 => Ok(Month::October),
            11 => Ok(Month::November),
            12 => Ok(Month::December),
            _ => Err("Month number must be between 1 and 12"),
        }
    }

    /// Return the numeric representation of the month (January = 1, ..., December = 12).
    pub fn number(&self) -> u8 {
        match self {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }

    /// Return the next month, cycling back to January after December.
    pub fn next(&self) -> Self {
        Month::from_number((self.number() % 12) + 1).unwrap()
    }

    /// Return the previous month, cycling back to December before January.
    pub fn previous(&self) -> Self {
        Month::from_number((self.number() + 10) % 12 + 1).unwrap()
    }
}


impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // Or a custom string if desired
    }
}

impl FromStr for Month {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "January"   => Ok(Month::January),
            "February"  => Ok(Month::February),
            "March"     => Ok(Month::March),
            "April"     => Ok(Month::April),
            "May"       => Ok(Month::May),
            "June"      => Ok(Month::June),
            "July"      => Ok(Month::July),
            "August"    => Ok(Month::August),
            "September" => Ok(Month::September),
            "October"   => Ok(Month::October),
            "November"  => Ok(Month::November),
            "December"  => Ok(Month::December),
            _           => Err("Invalid month string"),
        }
    }
}

#[cfg(test)]
mod month_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_month_iter() {
        let months: Vec<Month> = Month::iter().collect();
        assert_eq!(months.len(), 12);
        assert_eq!(months[0], Month::January);
        assert_eq!(months[11], Month::December);

        let set: HashSet<_> = months.iter().cloned().collect();
        assert_eq!(set.len(), 12);
    }

    #[test]
    fn test_month_from_number() {
        assert_eq!(Month::from_number(1), Ok(Month::January));
        assert_eq!(Month::from_number(12), Ok(Month::December));
        assert!(Month::from_number(0).is_err());
        assert!(Month::from_number(13).is_err());
    }

    #[test]
    fn test_month_number() {
        assert_eq!(Month::January.number(), 1);
        assert_eq!(Month::February.number(), 2);
        assert_eq!(Month::December.number(), 12);
    }

    #[test]
    fn test_month_from_str() {
        assert_eq!(Month::from_str("January"), Ok(Month::January));
        assert_eq!(Month::from_str("February"), Ok(Month::February));
        assert_eq!(Month::from_str("December"), Ok(Month::December));

        assert!(Month::from_str("").is_err());
        assert!(Month::from_str("Jan").is_err());
        assert!(Month::from_str("january").is_err());
        assert!(Month::from_str("December!").is_err());
    }

    #[test]
    fn test_month_display() {
        assert_eq!(Month::January.to_string(), "January");
        assert_eq!(Month::December.to_string(), "December");
    }

    #[test]
    fn test_month_navigation() {
        assert_eq!(Month::January.next(), Month::February);
        assert_eq!(Month::December.next(), Month::January);

        assert_eq!(Month::January.previous(), Month::December);
        assert_eq!(Month::March.previous(), Month::February);
    }
}
