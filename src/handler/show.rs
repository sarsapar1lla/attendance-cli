use itertools::Itertools;

use crate::{
    cli::show::Arguments, error::Result, model::Record, printer::record::Printer,
    repository::Repository,
};

pub fn show(args: &Arguments, repository: &dyn Repository, printer: &dyn Printer) -> Result<()> {
    let records = repository.get()?;

    let sorted: Vec<Record> = records
        .into_iter()
        .sorted_by(|a, b| a.created().cmp(b.created()).reverse())
        .filter(|r| match args.date() {
            None => true,
            Some(date) => r.key().date() == *date,
        })
        .collect();

    let truncated = if args.top() >= sorted.len() {
        sorted.as_slice()
    } else {
        &sorted.as_slice()[0..args.top()]
    };

    printer.print(truncated)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use uuid::Uuid;

    use crate::{
        error::Result,
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
        let first = record(1, 10);
        let second = record(1, 15);
        let fourth = record(1, 25);
        let third = record(1, 20);

        let args = Arguments::builder().top(2).build();
        let repository = InMemoryRepository::new(&[first, second, fourth.clone(), third.clone()]);
        let printer = InMemoryPrinter::new();

        show(&args, &repository, &printer).unwrap();

        assert_eq!(printer.printed(), vec![fourth, third])
    }

    #[test]
    fn prints_all_records_if_top_greater_than_record_size() {
        let first = record(1, 10);
        let second = record(1, 15);

        let args = Arguments::builder().top(3).build();
        let repository = InMemoryRepository::new(&[first.clone(), second.clone()]);
        let printer = InMemoryPrinter::new();

        show(&args, &repository, &printer).unwrap();

        assert_eq!(printer.printed(), vec![second, first])
    }

    #[test]
    fn prints_all_records_for_requested_date() {
        let first = record(1, 10);
        let second = record(1, 15);
        let third = record(2, 5);

        let args = Arguments::builder()
            .top(1)
            .date(jiff::civil::date(2025, 12, 1))
            .build();
        let repository = InMemoryRepository::new(&[first, second.clone(), third]);
        let printer = InMemoryPrinter::new();

        show(&args, &repository, &printer).unwrap();

        assert_eq!(printer.printed(), vec![second])
    }

    fn record(day: i8, minute: i8) -> Record {
        let created = jiff::civil::datetime(2025, 12, day, 10, minute, 0, 0);
        Record::builder()
            .id(Uuid::new_v4())
            .created(created.in_tz("Europe/London").unwrap().timestamp())
            .mode(Mode::Create)
            .record_type(RecordType::Office)
            .key(Key::FullDay(created.date()))
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

    impl Printer for InMemoryPrinter {
        fn print(&self, records: &[Record]) -> Result<()> {
            self.printed.lock().unwrap().append(&mut records.to_vec());
            Ok(())
        }
    }
}
