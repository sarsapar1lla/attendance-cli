use std::collections::HashMap;

use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use itertools::{Itertools, any};

use crate::{
    cli::summary::Arguments,
    error::Result,
    model::{Category, Mode, Record, RecordType, Summary},
    printer::SummaryPrinter,
    repository::Repository,
};

pub fn summary(
    args: &Arguments,
    repository: &dyn Repository,
    printer: &dyn SummaryPrinter,
    now_fn: fn() -> DateTime<Utc>,
) -> Result<()> {
    let number_of_months = args.months().unwrap_or(1);
    let months = last_n_months(number_of_months, now_fn);
    let records = repository.get()?;
    let filtered: Vec<Record> = records
        .into_iter()
        .filter(|r| record_in_months(r, &months))
        .collect();

    let keys = filtered
        .into_iter()
        .sorted_by(|a, b| a.key().cmp(b.key()).then(a.created().cmp(b.created())))
        .chunk_by(|r| r.key().clone());

    let deduplicated: Vec<Record> = keys
        .into_iter()
        .filter_map(|records| match records.1.last() {
            None => None,
            Some(record) if record.mode() == &Mode::Delete => None,
            Some(record) => Some(record),
        })
        .collect();

    let mut summaries = summarise(deduplicated);
    let empty: Vec<Summary> = months
        .into_iter()
        .filter(|month| !any(&summaries, |summary| summary.month() == month))
        .map(|month| summarise_month(month, &[]))
        .collect();

    summaries.extend(empty);

    summaries.sort_by(|a, b| a.month().cmp(b.month()).reverse());

    printer.print(&summaries);

    Ok(())
}

fn last_n_months(n: usize, now_fn: fn() -> DateTime<Utc>) -> Vec<NaiveDate> {
    let today = (now_fn)().date_naive();
    let this_month = month_of(&today);
    (0..n)
        .map(|months_back| {
            this_month
                .checked_sub_months(Months::new(u32::try_from(months_back).unwrap()))
                .unwrap()
        })
        .collect()
}

fn record_in_months(record: &Record, months: &[NaiveDate]) -> bool {
    let month = month_of(&record.key().date());
    months.contains(&month)
}

fn summarise(records: Vec<Record>) -> Vec<Summary> {
    let months = records.into_iter().chunk_by(|r| month_of(&r.key().date()));
    months
        .into_iter()
        .map(|month| summarise_month(month.0, month.1.collect_vec().as_slice()))
        .collect()
}

fn summarise_month(month: NaiveDate, records: &[Record]) -> Summary {
    let excluded: HashMap<NaiveDate, f32> = records
        .iter()
        .filter(|r| r.record_type() != &RecordType::Office)
        .map(|r| {
            (
                r.key().date(),
                if r.key().half_day() { 0.5f32 } else { 0.0f32 },
            )
        })
        .collect();

    let workdays = month
        .iter_days()
        .take_while(|date| date.month() == month.month())
        .filter(|date| Category::from(date) == Category::Workday)
        .map(|date| excluded.get(&date).unwrap_or(&1.0f32))
        .sum();

    let office_days = if records.is_empty() {
        0.0f32
    } else {
        records
            .iter()
            .filter(|r| r.record_type() == &RecordType::Office)
            .map(|_| 1.0f32)
            .sum()
    }
    .abs();

    let attendance: f32 = office_days / workdays;
    let attendance = (attendance * 1000.0).round() / 1000.0;

    Summary::builder()
        .month(month)
        .target_days(workdays * 0.50)
        .office_days(office_days)
        .workdays(workdays)
        .attendance(attendance)
        .build()
}

fn month_of(date: &NaiveDate) -> NaiveDate {
    date.with_day(1).expect("Every month has a first day")
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::{NaiveDateTime, NaiveTime};

    use crate::repository::test_utils::{FailingRepository, InMemoryRepository};

    use super::*;

    #[test]
    fn returns_error_if_cannot_access_repository() {
        let result = summary(
            &Arguments::builder().build(),
            &FailingRepository,
            &InMemoryPrinter::new(),
            Utc::now,
        );

        assert!(result.is_err())
    }

    #[test]
    fn summarises_latest_month_when_no_records() {
        let args = Arguments::builder().build();
        let repository = InMemoryRepository::new(&[]);
        let printer = InMemoryPrinter::new();

        summary(&args, &repository, &printer, now).unwrap();

        assert_eq!(
            printer.printed(),
            vec![
                Summary::builder()
                    .month(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap())
                    .target_days(10.5)
                    .office_days(0.0)
                    .workdays(21.0)
                    .attendance(0.0)
                    .build()
            ]
        )
    }

    // #[test]
    // fn summarises_latest_month() {
    //     let args = Arguments::builder().build();
    //     let repository = InMemoryRepository::new(&[Record::builder().build()]);
    //     let printer = InMemoryPrinter::new();

    //     summary(&args, &repository, &printer, now).unwrap();

    //     assert_eq!(
    //         printer.printed(),
    //         vec![
    //             Summary::builder()
    //                 .month(SummaryMonth::new(
    //                     NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()
    //                 ))
    //                 .office_days(0.0)
    //                 .workdays(21.0)
    //                 .attendance(0.0)
    //                 .build()
    //         ]
    //     )
    // }

    // #[test]
    // fn summarises_latest_n_months() {
    //     let args = Arguments::builder().months(2).build();
    //     let created = Utc::now();
    //     let repository = InMemoryRepository::new(&[Record::builder()
    //         .id(Uuid::new_v4())
    //         .created(created)
    //         .mode(Mode::Create)
    //         .record_type(RecordType::Office)
    //         .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 12).unwrap()))
    //         .build()]);
    //     let printer = InMemoryPrinter::new();

    //     summary(&args, &repository, &printer, now).unwrap();

    //     assert_eq!(printer.printed(), vec![])
    // }

    fn now() -> DateTime<Utc> {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
        .and_utc()
    }

    mod summarise_tests {
        use chrono::TimeZone;
        use uuid::Uuid;

        use crate::model::Key;

        use super::*;

        #[test]
        fn summarises_each_month() {
            let records = vec![record(9), record(10), record(11)];
            let actual = summarise(records);
            assert_eq!(
                actual,
                vec![
                    Summary::builder()
                        .month(NaiveDate::from_ymd_opt(2025, 9, 1).unwrap())
                        .target_days(11.0)
                        .office_days(1.0)
                        .workdays(22.0)
                        .attendance(0.045)
                        .build(),
                    Summary::builder()
                        .month(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap())
                        .target_days(11.5)
                        .office_days(1.0)
                        .workdays(23.0)
                        .attendance(0.043)
                        .build(),
                    Summary::builder()
                        .month(NaiveDate::from_ymd_opt(2025, 11, 1).unwrap())
                        .target_days(10.0)
                        .office_days(1.0)
                        .workdays(20.0)
                        .attendance(0.05)
                        .build()
                ]
            )
        }

        fn record(month: u32) -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created(Utc.with_ymd_and_hms(2025, 10, 31, 10, 0, 0).unwrap())
                .mode(Mode::Create)
                .record_type(RecordType::Office)
                .key(Key::FullDay(
                    NaiveDate::from_ymd_opt(2025, month, 1).unwrap(),
                ))
                .build()
        }
    }

    mod summarise_month_tests {
        use chrono::TimeZone;
        use uuid::Uuid;

        use crate::model::Key;

        use super::*;

        #[test]
        fn counts_office_days() {
            let month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
            let records = vec![
                record(1, RecordType::Office),
                record(2, RecordType::Office),
                record(6, RecordType::Office),
            ];
            let actual = summarise_month(month, &records);
            assert_eq!(actual.office_days(), 3.0)
        }

        #[test]
        fn counts_workdays() {
            let month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
            let actual = summarise_month(month, &[]);
            assert_eq!(actual.workdays(), 23.0)
        }

        #[test]
        fn counts_workdays_including_exclusions() {
            let month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
            let records = vec![
                record(1, RecordType::WorkingFromHome),
                record(2, RecordType::AnnualLeave),
                record(6, RecordType::Sick),
            ];
            let actual = summarise_month(month, &records);
            assert_eq!(actual.workdays(), 20.0)
        }

        #[test]
        fn counts_workdays_excluding_bank_holidays() {
            let month = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
            let actual = summarise_month(month, &[]);
            assert_eq!(actual.workdays(), 21.0)
        }

        #[test]
        fn calculates_attendance() {
            let month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
            let records = vec![
                record(1, RecordType::Office),
                record(2, RecordType::Office),
                record(6, RecordType::Office),
            ];
            let actual = summarise_month(month, &records);
            assert_eq!(actual.attendance(), 0.13)
        }

        #[test]
        fn calculates_target_days() {
            let month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
            let actual = summarise_month(month, &[]);
            assert_eq!(actual.target_days(), 11.5)
        }

        fn record(day: u32, record_type: RecordType) -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created(Utc.with_ymd_and_hms(2025, 10, 31, 10, 0, 0).unwrap())
                .mode(Mode::Create)
                .record_type(record_type)
                .key(Key::FullDay(
                    NaiveDate::from_ymd_opt(2025, 10, day).unwrap(),
                ))
                .build()
        }
    }

    struct InMemoryPrinter {
        printed: Mutex<Vec<Summary>>,
    }

    impl InMemoryPrinter {
        fn new() -> Self {
            Self {
                printed: Mutex::new(Vec::new()),
            }
        }

        fn printed(&self) -> Vec<Summary> {
            self.printed.lock().unwrap().to_vec()
        }
    }

    impl SummaryPrinter for InMemoryPrinter {
        fn print(&self, summaries: &[Summary]) {
            self.printed.lock().unwrap().append(&mut summaries.to_vec());
        }
    }
}
