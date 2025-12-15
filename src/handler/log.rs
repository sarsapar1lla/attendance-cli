use chrono::Utc;
use uuid::Uuid;

use crate::{
    cli::{self, Exclusion},
    error::{Error, Result},
    handler::day,
    model::{Category, Record, RecordType, State},
    repository::Repository,
};

pub fn log(args: &cli::LogArgs, repository: &dyn Repository) -> Result<()> {
    let records = repository.get()?;
    let record = record_from(args);

    let day_category = day::category(record.date());

    match day_category {
        Category::BankHoliday | Category::Weekend(_) => {
            Err(Error::NotAWorkday(record.date().to_owned(), day_category))
        }
        Category::Workday => Ok(()),
    }?;

    match (records.contains(record.date()), args.state()) {
        (false, State::Create) => repository.add(record),
        (true, State::Append | State::Delete) => repository.add(record),
        (true, State::Create) => Err(Error::RecordExistsForDate(record.date().to_owned())),
        (false, State::Append) => Err(Error::NoRecordToAppend(record.date().to_owned())),
        (false, State::Delete) => Err(Error::NoRecordToDelete(record.date().to_owned())),
    }
}

fn record_from(args: &cli::LogArgs) -> Record {
    let created = Utc::now();
    let record_type = args
        .exclusion()
        .map_or(RecordType::Office, |e| RecordType::from(e.to_owned()));

    Record::builder()
        .id(Uuid::new_v4())
        .created(created)
        .state(args.state())
        .record_type(record_type)
        .date(args.date().cloned().unwrap_or_else(|| created.date_naive()))
        .half_day(args.half_day())
        .maybe_description(args.description().cloned())
        .build()
}

impl From<Exclusion> for RecordType {
    fn from(value: Exclusion) -> Self {
        match value {
            Exclusion::WorkingFromHome => RecordType::WorkingFromHome,
            Exclusion::AnnualLeave => RecordType::AnnualLeave,
            Exclusion::Sick => RecordType::Sick,
            Exclusion::Other => RecordType::Other,
        }
    }
}
