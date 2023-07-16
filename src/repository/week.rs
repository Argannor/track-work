use std::fmt::Display;

use chrono::{Datelike, DateTime, NaiveDate, TimeZone};

/// Trait to abstract over different Date-related types that need to be converted to an ISO-Week
/// string
pub trait Week {
    /// converts this type to a string representation of the format `YYYY-WW` (e.g. 2023-05)
    fn to_week(&self) -> String;
}

impl <Tz: TimeZone> Week for DateTime<Tz> where Tz::Offset: Display {
    fn to_week(&self) -> String {
        format!("{}", self.format("%Y-%U"))
    }
}

impl Week for NaiveDate {
    fn to_week(&self) -> String {
        let week = self.iso_week();
        format!("{:4}-{:2}", week.year(), week.week())
    }
}