crate::ix!();

/// Enumerates the four meteorological seasons.
///
#[derive(EnumIter,Hash,Copy,Debug,PartialEq,Eq,Clone,Serialize,Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Return the season for a given month.
    pub fn from_month(m: Month) -> Self {
        match m {
            Month::March | Month::April | Month::May => Season::Spring,
            Month::June | Month::July | Month::August => Season::Summer,
            Month::September | Month::October | Month::November => Season::Autumn,
            Month::December | Month::January | Month::February => Season::Winter,
        }
    }

    /// Return a vector of all months belonging to this season.
    pub fn months(&self) -> Vec<Month> {
        match self {
            Season::Spring => vec![Month::March, Month::April, Month::May],
            Season::Summer => vec![Month::June, Month::July, Month::August],
            Season::Autumn => vec![Month::September, Month::October, Month::November],
            Season::Winter => vec![Month::December, Month::January, Month::February],
        }
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // or a custom string if desired
    }
}

impl FromStr for Season {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Spring" => Ok(Season::Spring),
            "Summer" => Ok(Season::Summer),
            "Autumn" => Ok(Season::Autumn),
            "Winter" => Ok(Season::Winter),
            _ => Err("Invalid season string"),
        }
    }
}

#[cfg(test)]
mod season_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_season_from_month() {
        // Spring: Mar, Apr, May
        for m in [Month::March, Month::April, Month::May].iter() {
            assert_eq!(Season::from_month(*m), Season::Spring);
        }

        // Summer: Jun, Jul, Aug
        for m in [Month::June, Month::July, Month::August].iter() {
            assert_eq!(Season::from_month(*m), Season::Summer);
        }

        // Autumn: Sep, Oct, Nov
        for m in [Month::September, Month::October, Month::November].iter() {
            assert_eq!(Season::from_month(*m), Season::Autumn);
        }

        // Winter: Dec, Jan, Feb
        for m in [Month::December, Month::January, Month::February].iter() {
            assert_eq!(Season::from_month(*m), Season::Winter);
        }
    }

    #[test]
    fn test_season_iter() {
        let seasons: Vec<Season> = Season::iter().collect();
        assert_eq!(seasons, vec![Season::Spring, Season::Summer, Season::Autumn, Season::Winter]);

        // Also check uniqueness
        let set: HashSet<_> = seasons.iter().cloned().collect();
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn test_season_display() {
        assert_eq!(Season::Spring.to_string(), "Spring");
        assert_eq!(Season::Summer.to_string(), "Summer");
        assert_eq!(Season::Autumn.to_string(), "Autumn");
        assert_eq!(Season::Winter.to_string(), "Winter");
    }

    #[test]
    fn test_season_from_str() {
        assert_eq!(Season::from_str("Spring"), Ok(Season::Spring));
        assert_eq!(Season::from_str("Summer"), Ok(Season::Summer));
        assert_eq!(Season::from_str("Autumn"), Ok(Season::Autumn));
        assert_eq!(Season::from_str("Winter"), Ok(Season::Winter));

        assert!(Season::from_str("sprIng").is_err());
        assert!(Season::from_str("Fall").is_err());
        assert!(Season::from_str("WINTER123").is_err());
        assert!(Season::from_str("").is_err());
    }

    #[test]
    fn test_season_months() {
        let spring_months = Season::Spring.months();
        assert_eq!(spring_months, vec![Month::March, Month::April, Month::May]);

        let summer_months = Season::Summer.months();
        assert_eq!(summer_months, vec![Month::June, Month::July, Month::August]);

        let autumn_months = Season::Autumn.months();
        assert_eq!(autumn_months, vec![Month::September, Month::October, Month::November]);

        let winter_months = Season::Winter.months();
        assert_eq!(winter_months, vec![Month::December, Month::January, Month::February]);
    }
}
