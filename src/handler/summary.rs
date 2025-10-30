use std::sync::LazyLock;

use chrono::{Datelike, Months, NaiveDate, Utc, Weekday};
use itertools::{Itertools, any};

use crate::{
    cli::{self},
    error::Result,
    model::{Record, RecordType, State, Summary, SummaryMonth},
    printer::SummaryPrinter,
    repository::Repository,
};

static BANK_HOLIDAYS: LazyLock<Vec<NaiveDate>> = LazyLock::new(|| {
    vec![
        NaiveDate::from_ymd_opt(2025, 12, 25).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 26).unwrap(),
    ]
});

pub fn summary(
    args: &cli::SummaryArgs,
    repository: &dyn Repository,
    printer: &dyn SummaryPrinter,
) -> Result<()> {
    let number_of_months = args.months().unwrap_or(1);
    let months = last_n_months(number_of_months);
    let records = repository.get()?;
    let filtered: Vec<Record> = records
        .into_iter()
        .filter(|r| record_in_months(r, &months))
        .collect();

    let dates = filtered
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(a.date(), b.date()).then(Ord::cmp(a.created(), b.created())))
        .chunk_by(|r| *r.date());

    let deduplicated: Vec<Record> = dates
        .into_iter()
        .filter_map(|records| match records.1.last() {
            None => None,
            Some(record) if record.state() == &State::Delete => None,
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

    printer.print(&summaries);

    Ok(())
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
    let excluded: Vec<&NaiveDate> = records
        .iter()
        .filter(|r| r.record_type() != &RecordType::Office)
        .map(Record::date)
        .collect();

    let workdays =
        NaiveDate::from_ymd_opt(month.year() as i32, month.month().number_from_month(), 1)
            .unwrap()
            .iter_days()
            .take_while(|date| date.month() == month.month().number_from_month())
            .filter(|date| ![Weekday::Sat, Weekday::Sun].contains(&date.weekday())) // Exclude weekends
            .filter(|date| !BANK_HOLIDAYS.contains(date)) // Not a bank holiday
            .filter(|date| !excluded.contains(&date)) // Apply exclusions
            .count();

    let office_days = records
        .iter()
        .filter(|r| r.record_type() == &RecordType::Office)
        .count();

    let attendance = (office_days as f32) / workdays as f32;

    Summary::builder()
        .month(month)
        .office_days(office_days)
        .workdays(workdays)
        .attendance(attendance)
        .build()
}

fn record_in_months(record: &Record, months: &[SummaryMonth]) -> bool {
    let month = SummaryMonth::new(*record.date());
    months.contains(&month)
}

fn last_n_months(n: usize) -> Vec<SummaryMonth> {
    let today = Utc::now()
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
