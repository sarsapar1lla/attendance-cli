use std::collections::HashMap;

use itertools::{Itertools, any};
use jiff::{
    ToSpan,
    civil::{Date, DateTime},
};

use crate::{
    cli::summary::Arguments,
    error::Result,
    model::{Category, Mode, Record, RecordType, Summary},
    printer::summary::Printer,
    repository::Repository,
};

pub fn summary(
    args: &Arguments,
    repository: &dyn Repository,
    printer: &dyn Printer,
    now_fn: fn() -> DateTime,
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
        .map(|month| summarise_month(&month, &[]))
        .collect();

    summaries.extend(empty);
    summaries.sort_by(|a, b| a.month().cmp(b.month()).reverse());
    printer.print(&summaries)
}

fn last_n_months(n: usize, now_fn: fn() -> DateTime) -> Vec<Date> {
    let today = (now_fn)().date();
    let this_month = today.first_of_month();

    this_month.series(-1.months()).take(n).collect()
}

fn record_in_months(record: &Record, months: &[Date]) -> bool {
    let month = record.key().date().first_of_month();
    months.contains(&month)
}

fn summarise(records: Vec<Record>) -> Vec<Summary> {
    let months = records
        .into_iter()
        .chunk_by(|r| r.key().date().first_of_month());
    months
        .into_iter()
        .map(|month| summarise_month(&month.0, month.1.collect_vec().as_slice()))
        .collect()
}

fn summarise_month(month: &Date, records: &[Record]) -> Summary {
    let excluded: HashMap<Date, f32> = records
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
        .series(1.days())
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
        .month(month.to_owned())
        .target_days(workdays * 0.50)
        .office_days(office_days)
        .workdays(workdays)
        .attendance(attendance)
        .build()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use jiff::Zoned;

    use crate::repository::test_utils::{FailingRepository, InMemoryRepository};

    use super::*;

    #[test]
    fn returns_error_if_cannot_access_repository() {
        let result = summary(
            &Arguments::builder().json(false).build(),
            &FailingRepository,
            &InMemoryPrinter::new(),
            || Zoned::now().datetime(),
        );

        assert!(result.is_err())
    }

    #[test]
    fn summarises_latest_month_when_no_records() {
        let args = Arguments::builder().json(false).build();
        let repository = InMemoryRepository::new(&[]);
        let printer = InMemoryPrinter::new();

        summary(&args, &repository, &printer, now).unwrap();

        assert_eq!(
            printer.printed(),
            vec![
                Summary::builder()
                    .month(jiff::civil::date(2025, 12, 1))
                    .target_days(10.5)
                    .office_days(0.0)
                    .workdays(21.0)
                    .attendance(0.0)
                    .build()
            ]
        )
    }

    fn now() -> DateTime {
        jiff::civil::datetime(2025, 12, 3, 10, 0, 0, 0)
    }

    mod summarise_tests {
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
                        .month(jiff::civil::date(2025, 9, 1))
                        .target_days(11.0)
                        .office_days(1.0)
                        .workdays(22.0)
                        .attendance(0.045)
                        .build(),
                    Summary::builder()
                        .month(jiff::civil::date(2025, 10, 1))
                        .target_days(11.5)
                        .office_days(1.0)
                        .workdays(23.0)
                        .attendance(0.043)
                        .build(),
                    Summary::builder()
                        .month(jiff::civil::date(2025, 11, 1))
                        .target_days(10.0)
                        .office_days(1.0)
                        .workdays(20.0)
                        .attendance(0.05)
                        .build()
                ]
            )
        }

        fn record(month: i8) -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created(
                    jiff::civil::datetime(2025, 10, 31, 10, 0, 0, 0)
                        .in_tz("Europe/London")
                        .unwrap()
                        .timestamp(),
                )
                .mode(Mode::Create)
                .record_type(RecordType::Office)
                .key(Key::FullDay(jiff::civil::date(2025, month, 1)))
                .build()
        }
    }

    mod summarise_month_tests {
        use uuid::Uuid;

        use crate::model::Key;

        use super::*;

        #[test]
        fn counts_office_days() {
            let month = jiff::civil::date(2025, 10, 1);
            let records = vec![
                record(1, RecordType::Office),
                record(2, RecordType::Office),
                record(6, RecordType::Office),
            ];
            let actual = summarise_month(&month, &records);
            assert_eq!(actual.office_days(), 3.0)
        }

        #[test]
        fn counts_workdays() {
            let month = jiff::civil::date(2025, 10, 1);
            let actual = summarise_month(&month, &[]);
            assert_eq!(actual.workdays(), 23.0)
        }

        #[test]
        fn counts_workdays_including_exclusions() {
            let month = jiff::civil::date(2025, 10, 1);
            let records = vec![
                record(1, RecordType::WorkingFromHome),
                record(2, RecordType::AnnualLeave),
                record(6, RecordType::Sick),
            ];
            let actual = summarise_month(&month, &records);
            assert_eq!(actual.workdays(), 20.0)
        }

        #[test]
        fn counts_workdays_excluding_bank_holidays() {
            let month = jiff::civil::date(2025, 12, 1);
            let actual = summarise_month(&month, &[]);
            assert_eq!(actual.workdays(), 21.0)
        }

        #[test]
        fn calculates_attendance() {
            let month = jiff::civil::date(2025, 10, 1);
            let records = vec![
                record(1, RecordType::Office),
                record(2, RecordType::Office),
                record(6, RecordType::Office),
            ];
            let actual = summarise_month(&month, &records);
            assert_eq!(actual.attendance(), 0.13)
        }

        #[test]
        fn calculates_target_days() {
            let month = jiff::civil::date(2025, 10, 1);
            let actual = summarise_month(&month, &[]);
            assert_eq!(actual.target_days(), 11.5)
        }

        fn record(day: i8, record_type: RecordType) -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created(
                    jiff::civil::datetime(2025, 10, 31, 10, 0, 0, 0)
                        .in_tz("Europe/London")
                        .unwrap()
                        .timestamp(),
                )
                .mode(Mode::Create)
                .record_type(record_type)
                .key(Key::FullDay(jiff::civil::date(2025, 10, day)))
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

    impl Printer for InMemoryPrinter {
        fn print(&self, summaries: &[Summary]) -> Result<()> {
            self.printed.lock().unwrap().append(&mut summaries.to_vec());
            Ok(())
        }
    }
}
