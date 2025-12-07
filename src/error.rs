use std::fmt::Display;

use chrono::NaiveDate;

#[derive(Debug)]
pub enum Error {
    Io(String),
    ReadFailure(String),
    WriteFailure(String),
    RecordExistsForDate(NaiveDate),
    NoRecordToAppend(NaiveDate),
    NoRecordToDelete(NaiveDate),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "Io error: {message}"),
            Self::ReadFailure(message) => write!(f, "Failed to read log file: {message}"),
            Self::WriteFailure(message) => write!(f, "Failed to write to log file: {message}"),
            Self::RecordExistsForDate(date) => write!(
                f,
                "Record exists for date '{date}'. To append an existing record, use the `--append` flag."
            ),
            Self::NoRecordToAppend(date) => write!(
                f,
                "Cannot append record for date '{date}' as no record exists."
            ),
            Self::NoRecordToDelete(date) => write!(
                f,
                "Cannot delete record for date '{date}' as no record exists."
            ),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
