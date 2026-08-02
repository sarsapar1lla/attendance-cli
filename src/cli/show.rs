use clap::Args;
use jiff::civil::Date;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    /// Limit the number of records returned
    #[arg(long, default_value = "10")]
    top: usize,

    /// Show records for a specific date
    #[arg(long)]
    date: Option<Date>,
}

impl Arguments {
    pub fn top(&self) -> usize {
        self.top
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }
}
