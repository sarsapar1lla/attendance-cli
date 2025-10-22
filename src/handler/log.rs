use chrono::Utc;
use uuid::Uuid;

use crate::{
    cli::{self, LogRecordType},
    model::{Record, RecordType},
    repository::Repository,
};

pub fn log(args: &cli::LogArgs, repository: &dyn Repository) {
    let record = record_from(args);
    repository.add(record);
}

fn record_from(args: &cli::LogArgs) -> Record {
    let created = Utc::now();
    let record_type = args
        .record_type()
        .map(|r| RecordType::from(r.to_owned()))
        .unwrap_or(RecordType::Office);

    Record::builder()
        .id(Uuid::new_v4())
        .created(created)
        .record_type(record_type)
        .date(args.date().cloned().unwrap_or_else(|| created.date_naive()))
        .maybe_description(args.description().cloned())
        .build()
}

impl From<LogRecordType> for RecordType {
    fn from(value: LogRecordType) -> Self {
        match value {
            LogRecordType::Office => RecordType::Office,
            LogRecordType::WorkingFromHome => RecordType::WorkingFromHome,
            LogRecordType::AnnualLeave => RecordType::AnnualLeave,
            LogRecordType::Sick => RecordType::Sick,
            LogRecordType::Other => RecordType::Other,
        }
    }
}
