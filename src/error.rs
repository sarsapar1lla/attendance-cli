use std::fmt::Display;

use jiff::civil::Date;

use crate::model::WeekendDay;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Error {
    Io(String),
    ReadFailure(String),
    WriteFailure(String),
    IsWeekend(Date, WeekendDay),
    IsBankHoliday(Date),
    RecordExistsForDate(Date),
    NoRecordToAppend(Date),
    NoRecordToDelete(Date),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "Io error: {message}."),
            Self::ReadFailure(message) => write!(f, "Failed to read log file: {message}."),
            Self::WriteFailure(message) => write!(f, "Failed to write to log file: {message}."),
            Self::IsBankHoliday(date) => {
                write!(f, "'{date}' is a bank holiday (England & Wales).")
            }
            Self::IsWeekend(date, day) => {
                write!(f, "'{date}' is on the weekend ({day:?}).")
            }
            Self::RecordExistsForDate(date) => write!(
                f,
                "Record exists for date '{date}'. To append an existing record, use `--mode append`."
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
