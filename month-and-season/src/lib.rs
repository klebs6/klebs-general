#[macro_use] mod imports; use imports::*;

x!{month}
x!{season}

#[cfg(test)]
mod month_and_season_tests {
    use super::*;

    #[test]
    fn test_month_season_round_trip() {
        // Check that for each month, converting to a season and back isn't broken.
        // (Though we don't have a "from_season" that returns a month, we can at least ensure seasons are consistent.)
        // Instead, we just ensure that each month matches the expected season above.
        let checks = [
            (Month::January,    Season::Winter),
            (Month::February,   Season::Winter),
            (Month::March,      Season::Spring),
            (Month::April,      Season::Spring),
            (Month::May,        Season::Spring),
            (Month::June,       Season::Summer),
            (Month::July,       Season::Summer),
            (Month::August,     Season::Summer),
            (Month::September,  Season::Autumn),
            (Month::October,    Season::Autumn),
            (Month::November,   Season::Autumn),
            (Month::December,   Season::Winter),
        ];

        for (m, s) in checks.iter() {
            assert_eq!(&Season::from_month(*m), s);
        }
    }
}
