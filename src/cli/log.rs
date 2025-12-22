use chrono::NaiveDate;
use clap::{ArgAction, Args, ValueEnum};

use crate::model;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long, value_enum)]
    exclusion: Option<Exclusion>,

    #[arg(long)]
    date: Option<NaiveDate>,

    #[arg(long, action = ArgAction::SetTrue)]
    half_day: bool,

    #[arg(long)]
    description: Option<String>,

    #[arg(long, value_enum, default_value_t)]
    mode: Mode,
}

impl Arguments {
    pub fn exclusion(&self) -> Option<&Exclusion> {
        self.exclusion.as_ref()
    }

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }

    pub fn half_day(&self) -> bool {
        self.half_day
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

#[derive(Debug, Clone, ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Exclusion {
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
pub enum Mode {
    Create,
    Append,
    Delete,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Create
    }
}
