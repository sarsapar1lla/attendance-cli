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
            .filter(|r| r.mode() != &Mode::Delete)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod records_tests {
        use chrono::{NaiveDate, TimeDelta, Utc};
        use uuid::Uuid;

        use crate::model::{HalfDay, RecordType};

        use super::*;

        mod full_day_tests {
            use chrono::NaiveDate;

            use super::*;

            #[test]
            fn returns_true_if_latest_record_for_day_is_active() {
                let key = Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_true_if_latest_record_for_part_of_day_is_active() {
                let key = Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 3).unwrap());
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_day_is_deleted() {
                let key = Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 2).unwrap());
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_day() {
                let key = Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 5).unwrap());
                assert!(!records().contains(&key))
            }
        }

        mod half_day_tests {
            use crate::model::HalfDay;

            use super::*;

            #[test]
            fn returns_true_if_latest_record_for_part_of_day_is_active() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                    half: HalfDay::Pm,
                };
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_true_if_latest_record_for_full_day_is_active() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
                    half: HalfDay::Am,
                };
                assert!(records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_part_of_day_is_deleted() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_latest_record_for_full_day_is_deleted() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 2).unwrap(),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_part_of_day() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 4).unwrap(),
                    half: HalfDay::Pm,
                };
                assert!(!records().contains(&key))
            }

            #[test]
            fn returns_false_if_no_record_for_day() {
                let key = Key::HalfDay {
                    date: NaiveDate::from_ymd_opt(2025, 12, 5).unwrap(),
                    half: HalfDay::Am,
                };
                assert!(!records().contains(&key))
            }
        }

        fn records() -> Records {
            let created = Utc::now();
            Records {
                records: vec![
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 2).unwrap()))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(
                            created
                                .checked_add_signed(TimeDelta::new(10, 0).unwrap())
                                .unwrap(),
                        )
                        .mode(Mode::Delete)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 2).unwrap()))
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                            half: HalfDay::Am,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(
                            created
                                .checked_add_signed(TimeDelta::new(10, 0).unwrap())
                                .unwrap(),
                        )
                        .mode(Mode::Delete)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                            half: HalfDay::Am,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(
                            created
                                .checked_add_signed(TimeDelta::new(20, 0).unwrap())
                                .unwrap(),
                        )
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                            half: HalfDay::Pm,
                        })
                        .build(),
                    Record::builder()
                        .id(Uuid::new_v4())
                        .created(created)
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::HalfDay {
                            date: NaiveDate::from_ymd_opt(2025, 12, 4).unwrap(),
                            half: HalfDay::Am,
                        })
                        .build(),
                ],
            }
        }
    }
}
