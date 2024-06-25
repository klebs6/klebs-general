crate::ix!();

#[derive(Debug,Clone)]
pub struct DateRange {
    start: NaiveDate,
    end:   Option<NaiveDate>,
}

impl DateRange {
    pub fn builder() -> DateRangeBuilder {
        DateRangeBuilder::default()
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> Option<NaiveDate> {
        self.end
    }
}

#[derive(Default)]
pub struct DateRangeBuilder {
    start: Option<NaiveDate>,
    end:   Option<NaiveDate>,
}

impl DateRangeBuilder {
    pub fn start(mut self, start: NaiveDate) -> Self {
        self.start = Some(start);
        self
    }

    pub fn end(mut self, end: Option<NaiveDate>) -> Self {
        self.end = end;
        self
    }

    pub fn build(self) -> DateRange {
        DateRange {
            start: self.start.expect("Start date is required"),
            end: self.end,
        }
    }
}

pub fn format_date_range(dates: &DateRange) -> String {
    let end_date = match dates.end() {
        Some(date) => date.to_string(),
        None => "Present".to_string(),
    };
    format!("{} - {}", dates.start(), end_date)
}

pub fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, ResumeBuilderError> {
    Ok(NaiveDate::from_ymd_opt(year, month, day).ok_or(ResumeBuilderError::CouldNotParseDate)?)
}

#[macro_export]
macro_rules! date {
    ($year:expr, $month:expr, $day:expr) => {
        date($year, $month, $day).unwrap()
    };
}

#[macro_export]
macro_rules! date_range {
    (start => $start:expr) => {
        DateRange::builder()
            .start(date!($start.0, $start.1, $start.2))
            .end(None)
            .build()
    };
    (start => $start:expr, end => $end:expr) => {
        DateRange::builder()
            .start(date!($start.0, $start.1, $start.2))
            .end(Some(date!($end.0, $end.1, $end.2)))
            .build()
    };
}
