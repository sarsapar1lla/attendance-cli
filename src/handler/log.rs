use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    cli::{self, Exclusion},
    error::{Error, Result},
    model::{Record, RecordType, State},
    repository::Repository,
};

pub fn log(args: &cli::LogArgs, repository: &dyn Repository) -> Result<()> {
    let records = repository.get()?;
    let record = record_from(args);

    let includes_date = includes_date(&records, record.date());

    if includes_date && !args.append() {
        Err(Error::RecordExistsForDate(record.date().to_owned()))
    } else {
        repository.add(record)
    }
}

fn includes_date(records: &[Record], date: &NaiveDate) -> bool {
    records.iter().any(|r| r.date() == date)
}

fn record_from(args: &cli::LogArgs) -> Record {
    let created = Utc::now();
    let state = if args.delete() {
        State::Delete
    } else {
        State::Create
    };
    let record_type = args
        .exclusion()
        .map_or(RecordType::Office, |e| RecordType::from(e.to_owned()));

    Record::builder()
        .id(Uuid::new_v4())
        .created(created)
        .state(state)
        .record_type(record_type)
        .date(args.date().copied().unwrap_or_else(|| created.date_naive()))
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
