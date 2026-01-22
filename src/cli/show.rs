use clap::Args;

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct Arguments {
    #[arg(long, default_value = "10")]
    top: usize,
}

impl Arguments {
    pub fn top(&self) -> usize {
        self.top
    }
}
