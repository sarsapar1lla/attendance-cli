use std::fmt::Display;

use bon::Builder;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum WeekendDay {
    Saturday,
    Sunday,
}

#[derive(Debug, PartialEq)]
pub enum Category {
    Workday,
    Weekend(WeekendDay),
    BankHoliday,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum Mode {
    Create,
    Append,
    Delete,
}

impl Display for Mode {
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

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub enum HalfDay {
    Am,
    Pm,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Key {
    FullDay(NaiveDate),
    HalfDay { date: NaiveDate, half: HalfDay },
}

impl Key {
    pub fn date(&self) -> NaiveDate {
        match *self {
            Key::FullDay(date) | Key::HalfDay { date, half: _ } => date,
        }
    }

    pub fn half_day(&self) -> bool {
        match *self {
            Key::FullDay(_) => false,
            Key::HalfDay { .. } => true,
        }
    }
}

#[derive(Debug, Clone, Builder, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Record {
    id: Uuid,
    created: DateTime<Utc>,
    mode: Mode,
    record_type: RecordType,
    key: Key,
    description: Option<String>,
}

impl Record {
    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn record_type(&self) -> &RecordType {
        &self.record_type
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

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
