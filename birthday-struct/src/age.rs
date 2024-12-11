crate::ix!();

impl Birthday {

    /// Calculates the age based on the birthday and current date.
    pub fn age(&self) -> Option<i32> {
        let today = Utc::now().date_naive(); // replaced Utc::today().naive_utc()

        let birth_date = NaiveDate::from_ymd_opt(*self.year(), *self.month(), *self.day())
            .expect("Birthday has invalid date");

        if birth_date > today {
            return None; // Birthday in the future
        }

        let mut age = today.year() - birth_date.year();

        // If today's month/day is before the birth month/day, subtract one year.
        if (today.month(), today.day()) < (birth_date.month(), birth_date.day()) {
            age -= 1;
        }

        Some(age)
    }

    /// Determines if the individual is above a certain age.
    pub fn is_above_age(&self, age: i32) -> bool {
        self.age().map_or(false, |actual_age| actual_age > age)
    }
}
