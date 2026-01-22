use itertools::Itertools;

use crate::{
    cli::show::Arguments, error::Result, model::Record, printer::RecordPrinter,
    repository::Repository,
};

pub fn show(
    args: &Arguments,
    repository: &dyn Repository,
    printer: &dyn RecordPrinter,
) -> Result<()> {
    let records = repository.get()?;

    let sorted: Vec<Record> = records
        .into_iter()
        .sorted_by(|a, b| a.created().cmp(b.created()).reverse())
        .collect();

    let truncated = if args.top() >= sorted.len() {
        sorted.as_slice()
    } else {
        &sorted.as_slice()[0..args.top()]
    };

    printer.print(truncated);
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::sync::Mutex;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use uuid::Uuid;

    use crate::{
        model::{Key, Mode, RecordType},
        repository::test_utils::{FailingRepository, InMemoryRepository},
    };

    use super::*;

    #[test]
    fn returns_error_if_cannot_access_repository() {
        let result = show(
            &Arguments::builder().top(10).build(),
            &FailingRepository,
            &InMemoryPrinter::new(),
        );
        assert!(result.is_err())
    }

    #[test]
    fn prints_top_n_records_with_latest_first() {
        let first = record(10);
        let second = record(15);
        let fourth = record(25);
        let third = record(20);

        let args = Arguments::builder().top(2).build();
        let repository = InMemoryRepository::new(&[first, second, fourth.clone(), third.clone()]);
        let printer = InMemoryPrinter::new();

        show(&args, &repository, &printer).unwrap();

        assert_eq!(printer.printed(), vec![fourth, third])
    }

    #[test]
    fn prints_all_records_if_top_greater_than_record_size() {
        let first = record(10);
        let second = record(15);

        let args = Arguments::builder().top(3).build();
        let repository = InMemoryRepository::new(&[first.clone(), second.clone()]);
        let printer = InMemoryPrinter::new();

        show(&args, &repository, &printer).unwrap();

        assert_eq!(printer.printed(), vec![second, first])
    }

    fn record(minute: u32) -> Record {
        let created = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
            NaiveTime::from_hms_opt(10, minute, 0).unwrap(),
        )
        .and_utc();
        Record::builder()
            .id(Uuid::new_v4())
            .created(created)
            .mode(Mode::Create)
            .record_type(RecordType::Office)
            .key(Key::FullDay(created.date_naive()))
            .build()
    }

    struct InMemoryPrinter {
        printed: Mutex<Vec<Record>>,
    }

    impl InMemoryPrinter {
        fn new() -> Self {
            Self {
                printed: Mutex::new(Vec::new()),
            }
        }

        fn printed(&self) -> Vec<Record> {
            self.printed.lock().unwrap().to_vec()
        }
    }

    impl RecordPrinter for InMemoryPrinter {
        fn print(&self, records: &[Record]) {
            self.printed.lock().unwrap().append(&mut records.to_vec());
        }
    }
}
