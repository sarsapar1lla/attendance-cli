use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

use crate::model::State;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Attendance logger", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn command(&self) -> &Command {
        &self.command
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Log attendance
    Log(LogArgs),

    /// Show log
    Show(ShowArgs),

    /// Summarise attendance
    Summary(SummaryArgs),
}

#[derive(Debug, Args)]
pub struct LogArgs {
    #[arg(long)]
    exclusion: Option<Exclusion>,

    #[arg(long)]
    date: Option<NaiveDate>,

    #[arg(long)]
    description: Option<String>,

    #[command(flatten)]
    flags: LogFlags,
}

impl LogArgs {
    pub fn exclusion(&self) -> Option<&Exclusion> {
        self.exclusion.as_ref()
    }

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn state(&self) -> State {
        match (self.flags.append, self.flags.delete) {
            (false, false) => State::Create,
            (true, false) => State::Append,
            (false, true) => State::Delete,
            _ => unreachable!("Clap makes this impossible"),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
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

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
struct LogFlags {
    #[arg(short, long, action = ArgAction::SetTrue)]
    append: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    delete: bool,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    #[arg(long)]
    top: Option<usize>,
}

impl ShowArgs {
    pub fn top(&self) -> Option<usize> {
        self.top
    }
}

#[derive(Debug, Args)]
pub struct SummaryArgs {
    #[arg(long)]
    months: Option<usize>,
}

impl SummaryArgs {
    pub fn months(&self) -> Option<usize> {
        self.months
    }
}
