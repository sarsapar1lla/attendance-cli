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
            Err(VarError::NotPresent) => home_dir().unwrap(),
            Err(error) => panic!("Invalid `{}` value: {}", XDG_DATA_HOME, error),
        };

        [data_directory, DIRECTORY.into()].iter().collect()
    }

    fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.directory).map_err(|e| Error::Io(e.to_string()))?;

        if let Ok(false) = std::fs::exists(&self.path) {
            File::create_new(&self.path).map_err(|e| Error::Io(e.to_string()))?;
        };

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

    fn get(&self) -> Result<Vec<Record>> {
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

        Ok(records)
    }
}
