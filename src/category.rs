use std::sync::LazyLock;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::model::{Category, WeekendDay};

const BANK_HOLIDAY_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/bank_holidays.json"
));

static BANK_HOLIDAYS: LazyLock<Vec<NaiveDate>> =
    LazyLock::new(|| serde_json::from_str(BANK_HOLIDAY_JSON).unwrap());

impl From<&NaiveDate> for Category {
    fn from(value: &NaiveDate) -> Self {
        match value {
            value if value.weekday() == Weekday::Sat => Category::Weekend(WeekendDay::Saturday),
            value if value.weekday() == Weekday::Sun => Category::Weekend(WeekendDay::Sunday),
            value if BANK_HOLIDAYS.contains(value) => Category::BankHoliday,
            _ => Category::Workday,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorises_saturday() {
        let date = date(13);
        assert_eq!(
            Category::from(&date),
            Category::Weekend(WeekendDay::Saturday)
        )
    }

    #[test]
    fn categorises_sunday() {
        let date = date(14);
        assert_eq!(Category::from(&date), Category::Weekend(WeekendDay::Sunday))
    }

    #[test]
    fn categorises_bank_holiday() {
        let date = date(25);
        assert_eq!(Category::from(&date), Category::BankHoliday)
    }

    #[test]
    fn categorises_workday() {
        let date = date(12);
        assert_eq!(Category::from(&date), Category::Workday)
    }

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, day).unwrap()
    }
}
