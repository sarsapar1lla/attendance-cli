use bon::Builder;
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Builder, Serialize)]
#[cfg_attr(test, derive(PartialEq, Clone))]
pub struct Summary {
    month: NaiveDate,
    target_days: f32,
    office_days: f32,
    workdays: f32,
    attendance: f32,
}

impl Summary {
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
