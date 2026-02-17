use std::{
    env::{self, VarError, home_dir},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    error::{Error, Result},
    model::Record,
};

const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
const DIRECTORY: &str = "attendance-cli";
const FILE_NAME: &str = "attendance.log";

pub trait Repository {
    fn add(&self, record: Record) -> Result<()>;

    fn get(&self) -> Result<Vec<Record>>;
}

pub struct FileRepository {
    directory: PathBuf,
    path: PathBuf,
}

impl FileRepository {
    pub fn new() -> Result<Self> {
        Self::directory().map(|dir| Self {
            directory: dir.clone(),
            path: [dir, FILE_NAME.into()].iter().collect(),
        })
    }

    fn directory() -> Result<PathBuf> {
        let data_directory = match env::var(XDG_DATA_HOME) {
            Ok(value) => PathBuf::from_str(&value).map_err(|e| Error::Io(e.to_string())),
            Err(VarError::NotPresent) => home_dir()
                .map(|home| [home, ".local/share".into()].iter().collect())
                .ok_or_else(|| Error::Io("Failed to construct repository file directory".into())),
            Err(error) => Err(Error::Io(format!(
                "Invalid `{XDG_DATA_HOME}` value: {error}"
            ))),
        }?;

        Ok([data_directory, DIRECTORY.into()].iter().collect())
    }

    fn log_file_exists(&self) -> Result<bool> {
        std::fs::exists(&self.path).map_err(|e| Error::Io(e.to_string()))
    }
}

impl Repository for FileRepository {
    fn add(&self, record: Record) -> Result<()> {
        if !self.log_file_exists()? {
            std::fs::create_dir_all(&self.directory).map_err(|e| Error::Io(e.to_string()))?;
            File::create_new(&self.path).map_err(|e| Error::Io(e.to_string()))?;
        }

        let file = File::options()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e.to_string()))?;

        let mut writer = BufWriter::new(file);
        write_to(&mut writer, &record)
    }

    fn get(&self) -> Result<Vec<Record>> {
        if !self.log_file_exists()? {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path).map_err(|e| Error::ReadFailure(e.to_string()))?;
        let mut reader = BufReader::new(file);
        read_from(&mut reader)
    }
}

fn read_from(buffer: &mut dyn BufRead) -> Result<Vec<Record>> {
    let mut records = Vec::new();

    for line in buffer.lines() {
        let line = line.map_err(|e| Error::Io(e.to_string()))?;
        let record: Record = serde_json::from_str(&line).map_err(|e| Error::Io(e.to_string()))?;
        records.push(record);
    }

    Ok(records)
}

fn write_to(buffer: &mut dyn Write, record: &Record) -> Result<()> {
    let content = serde_json::to_vec(&record).map_err(|e| Error::WriteFailure(e.to_string()))?;

    buffer
        .write_all(&content)
        .map_err(|e| Error::WriteFailure(e.to_string()))?;
    buffer
        .write(b"\n")
        .map_err(|e| Error::WriteFailure(e.to_string()))?;
    buffer
        .flush()
        .map_err(|e| Error::WriteFailure(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod read_from_tests {
        use chrono::NaiveDate;
        use uuid::Uuid;

        use crate::model::{Key, Mode, RecordType};

        use super::*;

        #[test]
        fn reads_records_from_buffer() {
            let input = File::open("./resources/test/repository/input.jsonl").unwrap();
            let mut buffer = BufReader::new(input);
            let records = read_from(&mut buffer).unwrap();
            assert_eq!(
                records,
                vec![
                    Record::builder()
                        .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                        .created("2025-12-01T10:00:00Z".parse().unwrap())
                        .mode(Mode::Create)
                        .record_type(RecordType::Office)
                        .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()))
                        .description("Went to the office".into())
                        .build(),
                    Record::builder()
                        .id(Uuid::parse_str("ac404846-bca7-4a5e-9291-8296d3be3a37").unwrap())
                        .created("2025-12-01T11:00:00Z".parse().unwrap())
                        .mode(Mode::Append)
                        .record_type(RecordType::AnnualLeave)
                        .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()))
                        .description("Went to the beach".into())
                        .build()
                ]
            )
        }

        #[test]
        fn returns_error_if_malformed_file() {
            let mut buffer = BufReader::new("Invalid".as_bytes());
            let result = read_from(&mut buffer);
            assert!(result.is_err())
        }
    }

    mod write_to_tests {
        use chrono::NaiveDate;
        use uuid::Uuid;

        use crate::model::{Key, Mode, RecordType};

        use super::*;

        #[test]
        fn writes_serialised_record_to_buffer() {
            let mut buffer = Vec::new();
            write_to(&mut buffer, &record()).unwrap();

            let actual = String::from_utf8(buffer).unwrap();
            let expected =
                std::fs::read_to_string("./resources/test/repository/output.jsonl").unwrap();
            assert_eq!(actual, expected)
        }

        #[test]
        fn returns_error_if_fails_to_write() {
            let result = write_to(&mut FailingWriter, &record());
            assert!(result.is_err())
        }

        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Failed!",
                ))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Failed!",
                ))
            }
        }

        fn record() -> Record {
            Record::builder()
                .id(Uuid::parse_str("0a766a52-c869-4be5-a695-4b258e2f2e87").unwrap())
                .created("2025-12-01T10:00:00Z".parse().unwrap())
                .mode(Mode::Create)
                .record_type(RecordType::Office)
                .key(Key::FullDay(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()))
                .description("Went to the office".into())
                .build()
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use chrono::NaiveDate;

    use crate::model::Mode;

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

        fn get(&self) -> Result<Vec<Record>> {
            Ok(self.records.lock().unwrap().to_vec())
        }
    }

    pub struct FailingRepository;

    impl Repository for FailingRepository {
        fn add(&self, _: Record) -> Result<()> {
            Err(Error::WriteFailure("Failure".into()))
        }

        fn get(&self) -> Result<Vec<Record>> {
            Err(Error::ReadFailure("Failure".into()))
        }
    }
}
