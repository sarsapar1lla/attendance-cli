use chrono::Utc;
use uuid::Uuid;

use crate::{
    cli::{self, log::Exclusion},
    error::{Error, Result},
    handler::day,
    model::{Category, Mode, Record, RecordType},
    repository::Repository,
};

pub fn log(args: &cli::log::Arguments, repository: &dyn Repository) -> Result<()> {
    let records = repository.get()?;
    let record = record_from(args);

    let day_category = day::category(record.date());

    match day_category {
        Category::BankHoliday => Err(Error::IsBankHoliday(record.date().to_owned())),
        Category::Weekend(day) => Err(Error::IsWeekend(record.date().to_owned(), day)),
        Category::Workday => match (records.contains(record.date()), args.mode()) {
            (false, Mode::Create) => repository.add(record),
            (true, Mode::Append | Mode::Delete) => repository.add(record),
            (true, Mode::Create) => Err(Error::RecordExistsForDate(record.date().to_owned())),
            (false, Mode::Append) => Err(Error::NoRecordToAppend(record.date().to_owned())),
            (false, Mode::Delete) => Err(Error::NoRecordToDelete(record.date().to_owned())),
        },
    }
}

fn record_from(args: &cli::log::Arguments) -> Record {
    let created = Utc::now();
    let record_type = args
        .exclusion()
        .map_or(RecordType::Office, |e| RecordType::from(e.to_owned()));

    Record::builder()
        .id(Uuid::new_v4())
        .created(created)
        .mode(args.mode())
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

    use chrono::NaiveDate;

    use crate::{
        cli::{log, log::Arguments},
        repository::tests::{FailingRepository, InMemoryRepository},
    };

    use super::*;

    #[test]
    fn returns_error_if_cannot_access_repository() {
        let args = Arguments::builder()
            .half_day(false)
            .mode(log::Mode::Create)
            .build();
        let result = log(&args, &FailingRepository);

        assert!(result.is_err())
    }

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
            let repository = InMemoryRepository::new(&[record(record_date, Mode::Create)]);

            let result = log(&args, &repository);
            assert_eq!(result.unwrap_err(), Error::RecordExistsForDate(record_date))
        }

        #[test]
        fn adds_record_to_repository_if_not_present() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[record(date(11), Mode::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(11), Mode::Create), (date(12), Mode::Create)]
            )
        }

        #[test]
        fn adds_record_to_repository_if_not_latest_state_is_delete() {
            let record_date = date(12);
            let args = args(record_date);
            let repository = InMemoryRepository::new(&[
                record(date(11), Mode::Create),
                record(record_date, Mode::Delete),
            ]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![
                    (date(11), Mode::Create),
                    (date(12), Mode::Delete),
                    (date(12), Mode::Create)
                ]
            )
        }

        fn args(record_date: NaiveDate) -> Arguments {
            Arguments::builder()
                .date(record_date)
                .half_day(false)
                .mode(log::Mode::Create)
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
            let repository = InMemoryRepository::new(&[record(record_date, Mode::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(12), Mode::Create), (date(12), Mode::Append)]
            )
        }

        fn args(record_date: NaiveDate) -> Arguments {
            Arguments::builder()
                .date(record_date)
                .half_day(false)
                .mode(log::Mode::Append)
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
            let repository = InMemoryRepository::new(&[record(record_date, Mode::Create)]);

            log(&args, &repository).unwrap();

            assert_eq!(
                repository.records(),
                vec![(date(12), Mode::Create), (date(12), Mode::Delete)]
            )
        }

        fn args(record_date: NaiveDate) -> Arguments {
            Arguments::builder()
                .date(record_date)
                .half_day(false)
                .mode(log::Mode::Delete)
                .build()
        }
    }

    #[test]
    fn maps_exclusion_to_record_type() {
        let exclusions = &[
            Exclusion::WorkingFromHome,
            Exclusion::AnnualLeave,
            Exclusion::Sick,
            Exclusion::Other,
        ];

        let record_types: Vec<RecordType> = exclusions
            .iter()
            .map(|e| RecordType::from(e.to_owned()))
            .collect();

        assert_eq!(
            record_types,
            vec![
                RecordType::WorkingFromHome,
                RecordType::AnnualLeave,
                RecordType::Sick,
                RecordType::Other
            ]
        )
    }

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, day).unwrap()
    }

    fn record(date: NaiveDate, mode: Mode) -> Record {
        Record::builder()
            .id(Uuid::new_v4())
            .created(Utc::now())
            .mode(mode)
            .record_type(RecordType::WorkingFromHome)
            .date(date)
            .half_day(false)
            .build()
    }
}
