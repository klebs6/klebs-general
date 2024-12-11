crate::ix!();

impl Into<ZodiacSign> for &Birthday {

    /// Returns the astrological zodiac sign.
    fn into(self) -> ZodiacSign {
        match (self.day(), self.month()) {
            (21..=31, 3)  | (1..=19, 4)  => ZodiacSign::Aries,
            (20..=30, 4)  | (1..=20, 5)  => ZodiacSign::Taurus,
            (21..=31, 5)  | (1..=20, 6)  => ZodiacSign::Gemini,
            (21..=30, 6)  | (1..=22, 7)  => ZodiacSign::Cancer,
            (23..=31, 7)  | (1..=22, 8)  => ZodiacSign::Leo,
            (23..=31, 8)  | (1..=22, 9)  => ZodiacSign::Virgo,
            (23..=30, 9)  | (1..=22, 10) => ZodiacSign::Libra,
            (23..=31, 10) | (1..=21, 11) => ZodiacSign::Scorpio,
            (22..=30, 11) | (1..=21, 12) => ZodiacSign::Sagittarius,
            (22..=31, 12) | (1..=19, 1)  => ZodiacSign::Capricorn,
            (20..=31, 1)  | (1..=18, 2)  => ZodiacSign::Aquarius,
            (19..=29, 2)  | (1..=20, 3)  => ZodiacSign::Pisces,
            (_, _) => unreachable!(), // This condition should be logically unreachable
        }
    }
}

#[cfg(test)]
mod birthday_zodiac_tests {
    use super::*;

    #[test]
    fn test_zodiac_signs() {

        // Define test cases for each zodiac sign, including boundary values.
        let test_data: Vec<(u32,u32,i32,ZodiacSign)> = vec![
            (21, 3,  1990, ZodiacSign::Aries),
            (19, 4,  1990, ZodiacSign::Aries),
            (20, 4,  1990, ZodiacSign::Taurus),
            (20, 5,  1990, ZodiacSign::Taurus),
            (21, 5,  1990, ZodiacSign::Gemini),
            (20, 6,  1990, ZodiacSign::Gemini),
            (21, 6,  1990, ZodiacSign::Cancer),
            (22, 7,  1990, ZodiacSign::Cancer),
            (23, 7,  1990, ZodiacSign::Leo),
            (22, 8,  1990, ZodiacSign::Leo),
            (23, 8,  1990, ZodiacSign::Virgo),
            (22, 9,  1990, ZodiacSign::Virgo),
            (23, 9,  1990, ZodiacSign::Libra),
            (22, 10, 1990, ZodiacSign::Libra),
            (23, 10, 1990, ZodiacSign::Scorpio),
            (21, 11, 1990, ZodiacSign::Scorpio),
            (22, 11, 1990, ZodiacSign::Sagittarius),
            (21, 12, 1990, ZodiacSign::Sagittarius),
            (22, 12, 1990, ZodiacSign::Capricorn),
            (19, 1,  1990, ZodiacSign::Capricorn),
            (20, 1,  1990, ZodiacSign::Aquarius),
            (18, 2,  1990, ZodiacSign::Aquarius),
            (19, 2,  1990, ZodiacSign::Pisces),
            (20, 3,  1990, ZodiacSign::Pisces),
        ];

        for (day, month, year, expected_sign) in test_data {

            let birthday = BirthdayBuilder::default()
                .day(day)
                .month(month)
                .year(year)
                .time_zone(Tz::UTC)
                .build()
                .expect("Failed to create Birthday instance");

            let sign: ZodiacSign = (&birthday).into();

            assert_eq!(sign, expected_sign, "Failed for {:?}-{:?}-{:?}", day, month, year);
        }
    }
}
