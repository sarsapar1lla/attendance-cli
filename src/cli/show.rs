use clap::Args;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long)]
    top: Option<usize>,
}

impl Arguments {
    pub fn top(&self) -> Option<usize> {
        self.top
    }
}
