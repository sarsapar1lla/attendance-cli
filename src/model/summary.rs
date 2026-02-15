use bon::Builder;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Builder)]
#[cfg_attr(test, derive(PartialEq, Clone))]
pub struct Summary {
    month: NaiveDate,
    target_days: f32,
    office_days: f32,
    workdays: f32,
    attendance: f32,
}

impl Summary {
    pub fn month_of(date: &NaiveDate) -> NaiveDate {
        date.with_day(1).expect("Every month has a first day")
    }

    pub fn month(&self) -> &NaiveDate {
        &self.month
    }

    pub fn target_days(&self) -> f32 {
        self.target_days
    }

    pub fn office_days(&self) -> f32 {
        self.office_days
    }

    pub fn workdays(&self) -> f32 {
        self.workdays
    }

    pub fn attendance(&self) -> f32 {
        self.attendance
    }
}
