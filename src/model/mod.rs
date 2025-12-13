use std::fmt::Display;

use bon::Builder;
use chrono::{DateTime, Datelike, Month, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum State {
    Create,
    Append,
    Delete,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum RecordType {
    Office,
    WorkingFromHome,
    AnnualLeave,
    Sick,
    Other,
}

impl Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Builder, Deserialize, Serialize)]
pub struct Record {
    id: Uuid,
    created: DateTime<Utc>,
    state: State,
    record_type: RecordType,
    date: NaiveDate,
    half_day: bool,
    description: Option<String>,
}

impl Record {
    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn record_type(&self) -> &RecordType {
        &self.record_type
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn half_day(&self) -> bool {
        self.half_day
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SummaryMonth {
    year: u32,
    month: Month,
}

impl SummaryMonth {
    pub fn new(date: NaiveDate) -> Self {
        let year = date.year_ce().1;
        let month = Month::try_from(u8::try_from(date.month()).unwrap()).unwrap();
        Self { year, month }
    }

    #[cfg(test)]
    pub fn from_parts(year: u32, month: Month) -> Self {
        Self { year, month }
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn month(&self) -> Month {
        self.month
    }
}

#[derive(Debug, Builder)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Summary {
    month: SummaryMonth,
    office_days: f32,
    workdays: f32,
    attendance: f32,
}

impl Summary {
    pub fn month(&self) -> &SummaryMonth {
        &self.month
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
