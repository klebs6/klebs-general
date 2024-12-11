crate::ix!();

/// Represents a Birthday
///
#[derive(Builder,Getters,Setters,Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct Birthday {

    /// The day of the month on which the birthday
    /// occurs. Range: [1, 31].
    ///
    day: u32,

    /// The month of the year on which the birthday
    /// occurs. Range: [1, 12].
    ///
    month: u32,

    /// The year in which the individual was born (Common
    /// Era).
    ///
    year: i32,

    /// The time zone in which the birthday is usually
    /// celebrated.
    ///
    time_zone: Tz,
}

impl Birthday {

    /// Determines if the birthday matches today's date.
    pub fn is_today(&self) -> bool {
        let today = Utc::now().date_naive();
        self.day == today.day() && self.month == today.month()
    }

    /// Computes the day of the week the person was born on.
    pub fn day_of_week(&self) -> Weekday {
        let birth_date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)
            .expect("Birthday has invalid date");
        birth_date.weekday()
    }
}

#[cfg(test)]
mod birthday_tests {
    use super::*;

    /// Test for Birthday creation.
    #[test]
    fn test_creation() {

        // Attempt to create a Birthday instance.
        let birthday = BirthdayBuilder::default()
            .day(15_u32)
            .month(8_u32)
            .year(1990)
            .time_zone(Tz::UTC)
            .build()
            .expect("failed to create Birthday");

        // Successful creation.
        assert_eq!(*birthday.day(), 15);
        assert_eq!(*birthday.month(), 8);
        assert_eq!(*birthday.year(), 1990);
        assert_eq!(*birthday.time_zone(), Tz::UTC);
    }

    /// Test for various Birthday methods.
    #[test]
    fn test_methods() {

        // Create a Birthday instance.
        let birthday = BirthdayBuilder::default()
            .day(15_u32)
            .month(8_u32)
            .year(1990)
            .time_zone(Tz::UTC)
            .build()
            .expect("failed to create Birthday");

        // Test if today is the birthday (likely false unless run on August 15).
        assert_eq!(birthday.is_today(), false);

        let sign: ZodiacSign = (&birthday).into();

        // Test zodiac sign (should be Leo for August 15).
        assert_eq!(sign, ZodiacSign::Leo);

        // Test day of the week (should be Wednesday for August 15, 1990).
        assert_eq!(birthday.day_of_week(), Weekday::Wed);

        // Test if the age is above 18 (should be true for a birthday in 1990).
        assert_eq!(birthday.is_above_age(18), true);
    }
}
