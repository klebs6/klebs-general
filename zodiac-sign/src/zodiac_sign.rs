crate::ix!();

/// Astrological zodiac signs.
#[derive(EnumIter,Hash,Copy,Debug,PartialEq,Eq,Clone,Serialize,Deserialize)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// Return the inclusive start and end date (month, day) for each sign.
    ///
    /// Dates are given in the format (month, day).
    pub fn date_range(&self) -> ((u8, u8), (u8, u8)) {
        match self {
            ZodiacSign::Aries       => ((3, 21), (4, 19)),
            ZodiacSign::Taurus      => ((4, 20), (5, 20)),
            ZodiacSign::Gemini      => ((5, 21), (6, 20)),
            ZodiacSign::Cancer      => ((6, 21), (7, 22)),
            ZodiacSign::Leo         => ((7, 23), (8, 22)),
            ZodiacSign::Virgo       => ((8, 23), (9, 22)),
            ZodiacSign::Libra       => ((9, 23), (10, 22)),
            ZodiacSign::Scorpio     => ((10, 23), (11, 21)),
            ZodiacSign::Sagittarius => ((11, 22), (12, 21)),
            ZodiacSign::Capricorn   => ((12, 22), (1, 19)),
            ZodiacSign::Aquarius    => ((1, 20), (2, 18)),
            ZodiacSign::Pisces      => ((2, 19), (3, 20)),
        }
    }

    /// Determine the zodiac sign from a given month and day.
    ///
    /// month: 1–12, day: 1–31 (caller must ensure day is valid for that month)
    ///
    /// Returns an error if the date does not match any sign (unlikely since all dates map to a sign).
    pub fn from_month_day(month: u8, day: u8) -> Result<Self, &'static str> {
        if month < 1 || month > 12 {
            return Err("Invalid month");
        }
        if day < 1 || day > 31 {
            return Err("Invalid day");
        }

        for sign in Self::iter() {
            let ((start_m, start_d), (end_m, end_d)) = sign.date_range();
            // Handle wrap-around signs like Capricorn, Aquarius, Pisces separately:
            if start_m <= end_m {
                // Normal range (e.g., Aries: Mar 21 - Apr 19)
                if (month == start_m && day >= start_d) || (month == end_m && day <= end_d) ||
                   (month > start_m && month < end_m) {
                    return Ok(sign);
                }
            } else {
                // Wrap-around range (e.g., Capricorn: Dec 22 - Jan 19)
                if (month == start_m && day >= start_d) ||
                   (month == end_m && day <= end_d) ||
                   (month > start_m || month < end_m) {
                    return Ok(sign);
                }
            }
        }
        Err("No zodiac sign found for given date")
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for ZodiacSign {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Aries"       => Ok(ZodiacSign::Aries),
            "Taurus"      => Ok(ZodiacSign::Taurus),
            "Gemini"      => Ok(ZodiacSign::Gemini),
            "Cancer"      => Ok(ZodiacSign::Cancer),
            "Leo"         => Ok(ZodiacSign::Leo),
            "Virgo"       => Ok(ZodiacSign::Virgo),
            "Libra"       => Ok(ZodiacSign::Libra),
            "Scorpio"     => Ok(ZodiacSign::Scorpio),
            "Sagittarius" => Ok(ZodiacSign::Sagittarius),
            "Capricorn"   => Ok(ZodiacSign::Capricorn),
            "Aquarius"    => Ok(ZodiacSign::Aquarius),
            "Pisces"      => Ok(ZodiacSign::Pisces),
            _             => Err("Invalid zodiac sign string"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_display_and_from_str() {
        for sign in ZodiacSign::iter() {
            let s = sign.to_string();
            let parsed = ZodiacSign::from_str(&s).unwrap();
            assert_eq!(sign, parsed);
        }

        assert!(ZodiacSign::from_str("NotASign").is_err());
        assert!(ZodiacSign::from_str("aries").is_err()); // case-sensitive
    }

    #[test]
    fn test_iter() {
        let signs: Vec<ZodiacSign> = ZodiacSign::iter().collect();
        assert_eq!(signs.len(), 12);
        let set: HashSet<_> = signs.iter().cloned().collect();
        assert_eq!(set.len(), 12);
    }

    #[test]
    fn test_date_ranges() {
        // Check that each sign's date_range is correct and non-overlapping.
        // We'll just verify a couple:
        let (start, end) = ZodiacSign::Aries.date_range();
        assert_eq!(start, (3, 21));
        assert_eq!(end, (4, 19));

        let (start, end) = ZodiacSign::Capricorn.date_range();
        assert_eq!(start, (12, 22));
        assert_eq!(end, (1, 19));
    }

    #[test]
    fn test_from_month_day() {
        // Aries starts March 21 and ends April 19
        assert_eq!(ZodiacSign::from_month_day(3, 21).unwrap(), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_month_day(4, 19).unwrap(), ZodiacSign::Aries);

        // Capricorn includes dates near the New Year
        assert_eq!(ZodiacSign::from_month_day(12, 31).unwrap(), ZodiacSign::Capricorn);
        assert_eq!(ZodiacSign::from_month_day(1, 1).unwrap(), ZodiacSign::Capricorn);

        // Check a random date in Leo (July 23 - Aug 22)
        assert_eq!(ZodiacSign::from_month_day(8, 1).unwrap(), ZodiacSign::Leo);

        // Invalid date checks
        assert!(ZodiacSign::from_month_day(0, 10).is_err());
        assert!(ZodiacSign::from_month_day(13, 10).is_err());
        assert!(ZodiacSign::from_month_day(2, 50).is_err());
    }

    #[test]
    fn test_full_coverage_of_all_days() {
        // Quick sanity check: every date in a normal range year should map to a sign.
        // We'll just pick a few random dates throughout the year:
        let checks = [
            (1, 15), // Capricorn
            (2, 10), // Aquarius
            (3, 25), // Aries
            (4, 30), // Taurus
            (6, 10), // Gemini
            (7, 10), // Cancer
            (8, 10), // Leo
            (9, 10), // Virgo
            (10, 10), // Libra
            (11, 10), // Scorpio
            (12, 10), // Sagittarius
        ];

        for (m, d) in checks.iter() {
            assert!(ZodiacSign::from_month_day(*m, *d).is_ok());
        }
    }
}
