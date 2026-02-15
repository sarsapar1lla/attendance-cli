use clap::{Args, ValueEnum};

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Arguments {
    shell: Shell,
}

impl Arguments {
    #[cfg(test)]
    pub fn new(shell: Shell) -> Self {
        Self { shell }
    }

    pub fn shell(&self) -> Shell {
        self.shell.clone()
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
}
