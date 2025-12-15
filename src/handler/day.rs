use std::sync::LazyLock;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::model::Category;

const BANK_HOLIDAY_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/bank_holidays.json"
));

static BANK_HOLIDAYS: LazyLock<Vec<NaiveDate>> =
    LazyLock::new(|| serde_json::from_str(BANK_HOLIDAY_JSON).unwrap());

pub fn category(date: &NaiveDate) -> Category {
    match date {
        date if date.weekday() == Weekday::Sat => Category::Weekend(Weekday::Sat),
        date if date.weekday() == Weekday::Sun => Category::Weekend(Weekday::Sun),
        date if BANK_HOLIDAYS.contains(date) => Category::BankHoliday,
        _ => Category::Workday,
    }
}
