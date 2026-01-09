use clap::Args;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long)]
    months: Option<usize>,
}

impl Arguments {
    pub fn months(&self) -> Option<usize> {
        self.months
    }
}
