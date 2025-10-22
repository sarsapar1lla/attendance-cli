use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

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
}

#[derive(Debug, Args)]
pub struct LogArgs {
    #[arg(long = "type")]
    record_type: Option<LogRecordType>,

    #[arg(long)]
    date: Option<NaiveDate>,

    #[arg(long)]
    description: Option<String>,

    #[arg(short, long, action = ArgAction::SetTrue)]
    append: bool,
}

impl LogArgs {
    pub fn record_type(&self) -> Option<&LogRecordType> {
        self.record_type.as_ref()
    }

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn append(&self) -> bool {
        self.append
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogRecordType {
    /// Working at the office
    Office,

    /// Authorised working from home
    WorkingFromHome,

    /// Annual leave
    AnnualLeave,

    /// Sick day
    Sick,

    /// Other
    Other,
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
