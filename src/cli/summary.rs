use clap::Args;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    /// Number of months to summarise
    ///
    /// When not provided only the current month will be summarised
    #[arg(long)]
    months: Option<usize>,
}

impl Arguments {
    pub fn months(&self) -> Option<usize> {
        self.months
    }
}
