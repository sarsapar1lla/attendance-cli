use chrono::NaiveDate;
use clap::Args;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long, default_value = "10")]
    top: usize,

    #[arg(long, help = "Show records for a specific date.")]
    date: Option<NaiveDate>,
}

impl Arguments {
    pub fn top(&self) -> usize {
        self.top
    }

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }
}
