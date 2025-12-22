use std::{
    env::{self, VarError, home_dir},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

use itertools::Itertools;

use crate::{
    error::{Error, Result},
    model::{Key, Mode, Record},
};

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
const DIRECTORY: &str = "attendance-cli";
const FILE_NAME: &str = "attendance.log";

pub struct Records {
    records: Vec<Record>,
}

impl Records {
    #[cfg(test)]
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn into_inner(self) -> Vec<Record> {
        self.records
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

pub trait Repository {
    fn add(&self, record: Record) -> Result<()>;

    fn get(&self) -> Result<Records>;
}

pub struct FileRepository {
    directory: PathBuf,
    path: PathBuf,
}

impl FileRepository {
    pub fn new() -> Self {
        let directory = Self::directory();
        Self {
            directory: directory.clone(),
            path: [directory, FILE_NAME.into()].iter().collect(),
        }
    }

    fn directory() -> PathBuf {
        let data_directory = match env::var(XDG_DATA_HOME) {
            Ok(value) => PathBuf::from_str(&value).unwrap(),
            Err(VarError::NotPresent) => home_dir()
                .map(|home| [home, ".local/share".into()].iter().collect())
                .unwrap(),
            Err(error) => panic!("Invalid `{XDG_DATA_HOME}` value: {error}"),
        };

        [data_directory, DIRECTORY.into()].iter().collect()
    }

    fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.directory).map_err(|e| Error::Io(e.to_string()))?;

        if let Ok(false) = std::fs::exists(&self.path) {
            File::create_new(&self.path).map_err(|e| Error::Io(e.to_string()))?;
        }

        Ok(())
    }
}

impl Repository for FileRepository {
    fn add(&self, record: Record) -> Result<()> {
        self.init()?;
        let file = File::options()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e.to_string()))?;

        let mut writer = BufWriter::new(file);

        let content =
            serde_json::to_vec(&record).map_err(|e| Error::WriteFailure(e.to_string()))?;

        writer
            .write_all(&content)
            .map_err(|e| Error::WriteFailure(e.to_string()))?;
        writer
            .write(b"\n")
            .map_err(|e| Error::WriteFailure(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| Error::WriteFailure(e.to_string()))?;
        Ok(())
    }

    fn get(&self) -> Result<Records> {
        self.init()?;
        let file = File::open(&self.path).map_err(|e| Error::ReadFailure(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| Error::Io(e.to_string()))?;
            let record: Record =
                serde_json::from_str(&line).map_err(|e| Error::Io(e.to_string()))?;
            records.push(record);
        }

        Ok(Records { records })
    }
}

#[cfg(test)]
pub mod test_utils {
    use chrono::NaiveDate;

    use super::*;

    pub struct InMemoryRepository {
        records: std::sync::Mutex<Vec<Record>>,
    }

    impl InMemoryRepository {
        pub fn new(records: &[Record]) -> Self {
            Self {
                records: std::sync::Mutex::new(records.to_vec()),
            }
        }

        pub fn records(&self) -> Vec<(NaiveDate, Mode)> {
            self.get()
                .unwrap()
                .into_inner()
                .into_iter()
                .map(|r| (r.key().date(), r.mode().to_owned()))
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

    pub struct FailingRepository;

    impl Repository for FailingRepository {
        fn add(&self, _: Record) -> Result<()> {
            Err(Error::WriteFailure("Failure".into()))
        }

        fn get(&self) -> Result<Records> {
            Err(Error::ReadFailure("Failure".into()))
        }
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
