use clap::{Args, ValueEnum};
use jiff::civil::Date;

use crate::model;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long = "type", value_enum, default_value_t)]
    record_type: RecordType,

    #[arg(
        long,
        help = "Date to log. If not provided, today's date will be used."
    )]
    date: Option<Date>,

    #[arg(long, value_enum)]
    half_day: Option<HalfDay>,

    #[arg(long)]
    description: Option<String>,

    #[arg(long, value_enum, default_value_t)]
    mode: Mode,
}

impl Arguments {
    pub fn record_type(&self) -> model::RecordType {
        match self.record_type {
            RecordType::Office => model::RecordType::Office,
            RecordType::WorkingFromHome => model::RecordType::WorkingFromHome,
            RecordType::AnnualLeave => model::RecordType::AnnualLeave,
            RecordType::Sick => model::RecordType::Sick,
            RecordType::Other => model::RecordType::Other,
        }
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn half_day(&self) -> Option<model::HalfDay> {
        match self.half_day {
            None => None,
            Some(HalfDay::Am) => Some(model::HalfDay::Am),
            Some(HalfDay::Pm) => Some(model::HalfDay::Pm),
        }
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn mode(&self) -> model::Mode {
        match self.mode {
            Mode::Create => model::Mode::Create,
            Mode::Append => model::Mode::Append,
            Mode::Delete => model::Mode::Delete,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RecordType {
    /// Office day
    #[default]
    Office,

    /// Authorised working from home
    #[clap(name = "wfh")]
    WorkingFromHome,

    /// Annual leave
    #[clap(name = "al")]
    AnnualLeave,

    /// Sick day
    Sick,

    /// Other
    Other,
}

#[derive(Debug, Clone, ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum HalfDay {
    Am,
    Pm,
}

#[derive(Debug, Clone, ValueEnum, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Mode {
    #[default]
    Create,
    Append,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod record_type_tests {
        use super::*;

        #[test]
        fn maps_office() {
            let args = args(RecordType::Office);
            assert_eq!(args.record_type(), model::RecordType::Office)
        }

        #[test]
        fn maps_working_from_home() {
            let args = args(RecordType::WorkingFromHome);
            assert_eq!(args.record_type(), model::RecordType::WorkingFromHome)
        }

        #[test]
        fn maps_annual_leave() {
            let args = args(RecordType::AnnualLeave);
            assert_eq!(args.record_type(), model::RecordType::AnnualLeave)
        }

        #[test]
        fn maps_annual_sick() {
            let args = args(RecordType::Sick);
            assert_eq!(args.record_type(), model::RecordType::Sick)
        }

        #[test]
        fn maps_annual_other() {
            let args = args(RecordType::Other);
            assert_eq!(args.record_type(), model::RecordType::Other)
        }

        fn args(record_type: RecordType) -> Arguments {
            Arguments::builder()
                .record_type(record_type)
                .mode(Mode::Create)
                .build()
        }
    }

    mod mode_tests {
        use super::*;

        #[test]
        fn maps_create() {
            let args = args(Mode::Create);
            assert_eq!(args.mode(), model::Mode::Create)
        }

        #[test]
        fn maps_append() {
            let args = args(Mode::Append);
            assert_eq!(args.mode(), model::Mode::Append)
        }

        #[test]
        fn maps_delete() {
            let args = args(Mode::Delete);
            assert_eq!(args.mode(), model::Mode::Delete)
        }

        fn args(mode: Mode) -> Arguments {
            Arguments::builder()
                .record_type(RecordType::Office)
                .mode(mode)
                .build()
        }
    }
}
