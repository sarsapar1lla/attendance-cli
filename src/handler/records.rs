use itertools::Itertools;

use crate::model::{Key, Mode, Record};

pub struct Records {
    records: Vec<Record>,
}

impl Records {
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn contains(&self, key: &Key) -> bool {
        let same_date = |r: &Record| r.key().date() == key.date();
        match *key {
            Key::FullDay(_) => self.contains_with_predicate(&same_date),
            Key::HalfDay { .. } => {
                if self.contains_with_predicate(&|r: &Record| same_date(r) && !r.key().half_day()) {
                    return true;
                }
                self.contains_with_predicate(&|r: &Record| r.key() == key)
            }
        }
    }

    fn contains_with_predicate(&self, predicate: &dyn Fn(&Record) -> bool) -> bool {
        let records_on_day: Vec<&Record> = self
            .records
            .iter()
            .filter(|r| predicate(r))
            .sorted_by_key(|r| r.created())
            .collect();

        records_on_day
            .last()
            .is_some_and(|r| r.mode() != &Mode::Delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod records_tests {
        use jiff::{Timestamp, ToSpan};
        use uuid::Uuid;

        use crate::model::{HalfDay, RecordType};

        use super::*;

        mod full_day_tests {

            use super::*;

            #[test]
            fn returns_true_if_latest_record_for_day_is_active() {
                let key = Key::FullDay(jiff::civil::date(2025, 12, 1));
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_true_if_latest_record_for_part_of_day_is_active() {
                let key = Key::FullDay(jiff::civil::date(2025, 12, 3));
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_day_is_deleted() {
                let key = Key::FullDay(jiff::civil::date(2025, 12, 2));
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_day() {
                let key = Key::FullDay(jiff::civil::date(2025, 12, 5));
                assert!(!records().contains(&key))
            }
        }

        mod half_day_tests {
            use crate::model::HalfDay;

            use super::*;

            #[test]
            fn returns_true_if_latest_record_for_part_of_day_is_active() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 3),
                    half: HalfDay::Pm,
                };
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_true_if_latest_record_for_full_day_is_active() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 1),
                    half: HalfDay::Am,
                };
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_part_of_day_is_deleted() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 3),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_full_day_is_deleted() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 2),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_part_of_day() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 4),
                    half: HalfDay::Pm,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_day() {
                let key = Key::HalfDay {
                    date: jiff::civil::date(2025, 12, 5),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }
        }

        fn records() -> Records {
            let created = Timestamp::now();
            Records {
                records: vec![
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(jiff::civil::date(2025, 12, 1)))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(jiff::civil::date(2025, 12, 2)))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created.checked_add(10.seconds()).unwrap())
                        .mode(Mode::Delete)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(jiff::civil::date(2025, 12, 2)))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: jiff::civil::date(2025, 12, 3),
                            half: HalfDay::Am,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created.checked_add(10.seconds()).unwrap())
                        .mode(Mode::Delete)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: jiff::civil::date(2025, 12, 3),
                            half: HalfDay::Am,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created.checked_add(20.seconds()).unwrap())
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: jiff::civil::date(2025, 12, 3),
                            half: HalfDay::Pm,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: jiff::civil::date(2025, 12, 4),
                            half: HalfDay::Am,
                        })
                        .build(),
                ],
            }
        }
    }
}
