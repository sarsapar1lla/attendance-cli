use std::fmt::Display;

use bon::Builder;
use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    FullDay(Date),
    HalfDay { date: Date, half: HalfDay },
}

impl Key {
    pub fn date(&self) -> Date {
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
    created: Timestamp,
    mode: Mode,
    record_type: RecordType,
    key: Key,
    description: Option<String>,
}

impl Record {
    pub fn created(&self) -> &Timestamp {
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
