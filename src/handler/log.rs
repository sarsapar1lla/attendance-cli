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
        Category::BankHoliday => Err(Error::IsBankHoliday(record.date().to_owned())),
        Category::Weekend(day) => Err(Error::IsWeekend(record.date().to_owned(), day)),
        Category::Workday => match (records.contains(record.date()), args.state()) {
            (false, State::Create) => repository.add(record),
            (true, State::Append | State::Delete) => repository.add(record),
            (true, State::Create) => Err(Error::RecordExistsForDate(record.date().to_owned())),
            (false, State::Append) => Err(Error::NoRecordToAppend(record.date().to_owned())),
            (false, State::Delete) => Err(Error::NoRecordToDelete(record.date().to_owned())),
        },
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::NaiveDate;

    use crate::{
        cli::{LogArgs, LogFlags},
        repository::Records,
    };

    use super::*;

    mod create_tests {
        use crate::model::WeekendDay;

        use super::*;

        #[test]
        fn returns_error_if_bank_holiday() {
            let record_date = date(25);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[]);

            let result = log(&args, &repository);
            assert_eq!(result.unwrap_err(), Error::IsBankHoliday(record_date))
        }

        #[test]
        fn returns_error_if_saturday() {
            let record_date = date(13);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[]);

            let result = log(&args, &repository);
            assert_eq!(
                result.unwrap_err(),
                Error::IsWeekend(record_date, WeekendDay::Saturday)
            )
        }

        #[test]
        fn returns_error_if_sunday() {
            let record_date = date(14);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[]);

            let result = log(&args, &repository);
            assert_eq!(
                result.unwrap_err(),
                Error::IsWeekend(record_date, WeekendDay::Sunday)
            )
        }

        #[test]
        fn returns_error_if_record_exits_for_day() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[record(record_date, State::Create)]);

            let result = log(&args, &repository);
            assert_eq!(result.unwrap_err(), Error::RecordExistsForDate(record_date))
        }

        #[test]
        fn adds_record_to_repository_if_not_present() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[record(date(11), State::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(11), State::Create), (date(12), State::Create)]
            )
        }

        #[test]
        fn adds_record_to_repository_if_not_latest_state_is_delete() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[
                record(date(11), State::Create),
                record(record_date, State::Delete),
            ]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![
                    (date(11), State::Create),
                    (date(12), State::Delete),
                    (date(12), State::Create)
                ]
            )
        }

        fn args(record_date: NaiveDate) -> LogArgs {
            LogArgs::builder()
                .date(record_date)
                .half_day(false)
                .flags(LogFlags::builder().append(false).delete(false).build())
                .build()
        }
    }

    mod append_tests {

        use super::*;

        #[test]
        fn returns_error_if_no_record_exists_for_date() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[]);

            let result = log(&args, &repository);
            assert_eq!(result.unwrap_err(), Error::NoRecordToAppend(record_date))
        }

        #[test]
        fn appends_existing_record() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[record(record_date, State::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(12), State::Create), (date(12), State::Append)]
            )
        }

        fn args(record_date: NaiveDate) -> LogArgs {
            LogArgs::builder()
                .date(record_date)
                .half_day(false)
                .flags(LogFlags::builder().append(true).delete(false).build())
                .build()
        }
    }

    mod delete_tests {

        use super::*;

        #[test]
        fn returns_error_if_no_record_exists_for_date() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[]);

            let result = log(&args, &repository);
            assert_eq!(result.unwrap_err(), Error::NoRecordToDelete(record_date))
        }

        #[test]
        fn deletes_existing_record() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[record(record_date, State::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(12), State::Create), (date(12), State::Delete)]
            )
        }

        fn args(record_date: NaiveDate) -> LogArgs {
            LogArgs::builder()
                .date(record_date)
                .half_day(false)
                .flags(LogFlags::builder().append(false).delete(true).build())
                .build()
        }
    }

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, day).unwrap()
    }

    fn record(date: NaiveDate, state: State) -> Record {
        Record::builder()
            .id(Uuid::new_v4())
            .created(Utc::now())
            .state(state)
            .record_type(RecordType::WorkingFromHome)
            .date(date)
            .half_day(false)
            .build()
    }

    struct InMemoryRepository {
        records: Mutex<Vec<Record>>,
    }

    impl InMemoryRepository {
        fn new(records: &[Record]) -> Self {
            Self {
                records: Mutex::new(records.to_vec()),
            }
        }

        fn records(&self) -> Vec<(NaiveDate, State)> {
            self.get()
                .unwrap()
                .into_inner()
                .into_iter()
                .map(|r| (r.date().to_owned(), r.state().to_owned()))
                .collect()
        }
    }

    impl Repository for InMemoryRepository {
        fn add(&self, record: Record) -> Result<()> {
            self.records.lock().unwrap().push(record);
            Ok(())
        }

        fn get(&self) -> Result<Records> {
            Ok(Records::new(self.records.lock().unwrap().to_vec()))
        }
    }
}
