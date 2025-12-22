use std::collections::HashMap;

use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use itertools::{Itertools, any};

use crate::{
    cli::summary::Arguments,
    error::Result,
    handler::day,
    model::{Category, Mode, Record, RecordType, Summary, SummaryMonth},
    printer::SummaryPrinter,
    repository::Repository,
};

pub struct Handler<'a> {
    repository: &'a dyn Repository,
    printer: &'a dyn SummaryPrinter,
    now_fn: fn() -> DateTime<Utc>,
}

impl<'a> Handler<'a> {
    pub fn new(repository: &'a dyn Repository, printer: &'a dyn SummaryPrinter) -> Handler<'a> {
        Self {
            repository,
            printer,
            now_fn: Utc::now,
        }
    }

    pub fn summary(&self, args: &Arguments) -> Result<()> {
        let number_of_months = args.months().unwrap_or(1);
        let months = self.last_n_months(number_of_months);
        let records = self.repository.get()?;
        let filtered: Vec<Record> = records
            .into_inner()
            .into_iter()
            .filter(|r| Handler::record_in_months(r, &months))
            .collect();

        let dates = filtered
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(a.date(), b.date()).then(Ord::cmp(a.created(), b.created())))
            .chunk_by(|r| *r.date());

        let deduplicated: Vec<Record> = dates
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

        summaries.sort_by(|a, b| Ord::cmp(a.month(), b.month()).reverse());

        self.printer.print(&summaries);

        Ok(())
    }

    fn last_n_months(&self, n: usize) -> Vec<SummaryMonth> {
        let today = (self.now_fn)()
            .date_naive()
            .with_day(1)
            .expect("Every month has a first day");
        (0..n)
            .map(|months_back| {
                today
                    .checked_sub_months(Months::new(months_back as u32))
                    .unwrap()
            })
            .map(SummaryMonth::new)
            .collect()
    }

    fn record_in_months(record: &Record, months: &[SummaryMonth]) -> bool {
        let month = SummaryMonth::new(*record.date());
        months.contains(&month)
    }
}

fn summarise(records: Vec<Record>) -> Vec<Summary> {
    let months = records
        .into_iter()
        .chunk_by(|r| SummaryMonth::new(*r.date()));
    months
        .into_iter()
        .map(|month| summarise_month(month.0, month.1.collect_vec().as_slice()))
        .collect()
}

fn summarise_month(month: SummaryMonth, records: &[Record]) -> Summary {
    let excluded: HashMap<&NaiveDate, f32> = records
        .iter()
        .filter(|r| r.record_type() != &RecordType::Office)
        .map(|r| (r.date(), if r.half_day() { 0.5 } else { 0.0 }))
        .collect();

    let workdays =
        NaiveDate::from_ymd_opt(month.year() as i32, month.month().number_from_month(), 1)
            .unwrap()
            .iter_days()
            .take_while(|date| date.month() == month.month().number_from_month())
            .filter(|date| day::category(date) == Category::Workday)
            .map(|date| excluded.get(&date).unwrap_or(&1.0))
            .sum();

    let office_days = records
        .iter()
        .filter(|r| r.record_type() == &RecordType::Office)
        .map(|r| if r.half_day() { 0.5 } else { 1.0 })
        .sum();

    let attendance: f32 = office_days / workdays;
    let attendance = (attendance * 1000.0).round() / 1000.0;

    Summary::builder()
        .month(month)
        .office_days(office_days)
        .workdays(workdays)
        .attendance(attendance)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod summarise_tests {
        use chrono::{Month, TimeZone};
        use uuid::Uuid;

        use super::*;

        #[test]
        fn summarises_each_month() {
            let records = vec![record(9), record(10), record(11)];
            let actual = summarise(records);
            assert_eq!(
                actual,
                vec![
                    Summary::builder()
                        .month(SummaryMonth::from_parts(2025, Month::September))
                        .office_days(1.0)
                        .workdays(22.0)
                        .attendance(0.045)
                        .build(),
                    Summary::builder()
                        .month(SummaryMonth::from_parts(2025, Month::October))
                        .office_days(1.0)
                        .workdays(23.0)
                        .attendance(0.043)
                        .build(),
                    Summary::builder()
                        .month(SummaryMonth::from_parts(2025, Month::November))
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
                .date(NaiveDate::from_ymd_opt(2025, month, 1).unwrap())
                .half_day(false)
                .build()
        }
    }

    mod summarise_month_tests {
        use chrono::TimeZone;
        use uuid::Uuid;

        use super::*;

        #[test]
        fn counts_office_days() {
            let month = SummaryMonth::new(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
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
            let month = SummaryMonth::new(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
            let actual = summarise_month(month, &[]);
            assert_eq!(actual.workdays(), 23.0)
        }

        #[test]
        fn counts_workdays_including_exclusions() {
            let month = SummaryMonth::new(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
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
            let month = SummaryMonth::new(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
            let actual = summarise_month(month, &[]);
            assert_eq!(actual.workdays(), 21.0)
        }

        #[test]
        fn calculates_attendance() {
            let month = SummaryMonth::new(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
            let records = vec![
                record(1, RecordType::Office),
                record(2, RecordType::Office),
                record(6, RecordType::Office),
            ];
            let actual = summarise_month(month, &records);
            assert_eq!(actual.attendance(), 0.13)
        }

        fn record(day: u32, record_type: RecordType) -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created(Utc.with_ymd_and_hms(2025, 10, 31, 10, 0, 0).unwrap())
                .mode(Mode::Create)
                .record_type(record_type)
                .date(NaiveDate::from_ymd_opt(2025, 10, day).unwrap())
                .half_day(false)
                .build()
        }
    }
}
