crate::ix!();

impl Birthday {

    /// Computes the number of days until the next birthday.
    pub fn days_until_next(&self) -> i64 {
        let today = Utc::now().date_naive();

        let this_year_birthday = NaiveDate::from_ymd_opt(
            today.year(), *self.month(), *self.day()
        ).expect("Invalid birthday date");

        if today <= this_year_birthday {
            return (this_year_birthday - today).num_days();
        }

        let next_year_birthday = NaiveDate::from_ymd_opt(
            today.year() + 1, *self.month(), *self.day()
        ).expect("Invalid birthday date");
        (next_year_birthday - today).num_days()
    }

    /// Computes the number of days since the last birthday.
    pub fn days_since_last(&self) -> i64 {
        let today = Utc::now().date_naive();

        let this_year_birthday = NaiveDate::from_ymd_opt(
            today.year(), *self.month(), *self.day()
        ).expect("Invalid birthday date");

        if today >= this_year_birthday {
            return (today - this_year_birthday).num_days();
        }

        let last_year_birthday = NaiveDate::from_ymd_opt(
            today.year() - 1, *self.month(), *self.day()
        ).expect("Invalid birthday date");
        (today - last_year_birthday).num_days()
    }
}

#[cfg(test)]
mod birthday_days_since_last_tests {
    use super::*;

    /// Test for days since the last birthday.
    #[test]
    fn test_days_since_last() {

        // Create a Birthday instance.
        let birthday = BirthdayBuilder::default()
            .day(15_u32)
            .month(8_u32)
            .year(1990)
            .time_zone(Tz::UTC)
            .build()
            .expect("failed to create Birthday");

        // Fetch the days since the last birthday.
        let days_since_last = birthday.days_since_last();

        // Ensure that the result is non-negative.
        assert!(days_since_last >= 0);
    }
}
