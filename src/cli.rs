use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

use crate::model::State;

#[derive(Debug, Parser)]
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

#[derive(Debug, Subcommand)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Command {
    /// Log attendance
    Log(LogArgs),

    /// Show log
    Show(ShowArgs),

    /// Summarise attendance
    Summary(SummaryArgs),
}

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
pub struct LogArgs {
    #[arg(long)]
    exclusion: Option<Exclusion>,

    #[arg(long)]
    date: Option<NaiveDate>,

    #[arg(long, action = ArgAction::SetTrue)]
    half_day: bool,

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

    pub fn half_day(&self) -> bool {
        self.half_day
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

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq, bon::Builder))]
#[group(required = false, multiple = false)]
pub struct LogFlags {
    #[arg(short, long, action = ArgAction::SetTrue)]
    append: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    delete: bool,
}

#[derive(Debug, Args)]
#[cfg_attr(test, derive(PartialEq))]
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
#[cfg_attr(test, derive(PartialEq))]
pub struct SummaryArgs {
    #[arg(long)]
    months: Option<usize>,
}

impl SummaryArgs {
    pub fn months(&self) -> Option<usize> {
        self.months
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod log_tests {
        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "log"]).unwrap();
            let expected = LogArgs::builder()
                .half_day(false)
                .flags(LogFlags {
                    append: false,
                    delete: false,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        mod exclusion_tests {
            use super::*;

            #[test]
            fn parses_with_exclusion() {
                let args =
                    Cli::try_parse_from(&["attendance", "log", "--exclusion", "wfh"]).unwrap();
                let expected = LogArgs::builder()
                    .exclusion(Exclusion::WorkingFromHome)
                    .half_day(false)
                    .flags(LogFlags {
                        append: false,
                        delete: false,
                    })
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn returns_error_if_not_valid_exclusion() {
                let result = Cli::try_parse_from(&["attendance", "log", "--exclusion", "invalid"]);
                assert!(result.is_err())
            }
        }

        mod date_tests {
            use super::*;

            #[test]
            fn parses_with_date() {
                let args =
                    Cli::try_parse_from(&["attendance", "log", "--date", "2025-12-01"]).unwrap();
                let expected = LogArgs::builder()
                    .date(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap())
                    .half_day(false)
                    .flags(LogFlags {
                        append: false,
                        delete: false,
                    })
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn returns_error_if_not_valid_date() {
                let result = Cli::try_parse_from(&["attendance", "log", "--date", "invalid"]);
                assert!(result.is_err())
            }
        }

        #[test]
        fn parses_with_half_day() {
            let args = Cli::try_parse_from(&["attendance", "log", "--half-day"]).unwrap();
            let expected = LogArgs::builder()
                .half_day(true)
                .flags(LogFlags {
                    append: false,
                    delete: false,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_description() {
            let args =
                Cli::try_parse_from(&["attendance", "log", "--description", "Party!"]).unwrap();
            let expected = LogArgs::builder()
                .description("Party!".into())
                .half_day(false)
                .flags(LogFlags {
                    append: false,
                    delete: false,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_append() {
            let args = Cli::try_parse_from(&["attendance", "log", "--append"]).unwrap();
            let expected = LogArgs::builder()
                .half_day(false)
                .flags(LogFlags {
                    append: true,
                    delete: false,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_delete() {
            let args = Cli::try_parse_from(&["attendance", "log", "--delete"]).unwrap();
            let expected = LogArgs::builder()
                .half_day(false)
                .flags(LogFlags {
                    append: false,
                    delete: true,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn returns_error_if_append_and_delete_passed() {
            let result = Cli::try_parse_from(&["attendance", "log", "--append", "--delete"]);
            assert!(result.is_err())
        }

        #[test]
        fn parses_with_everything() {
            let args = Cli::try_parse_from(&[
                "attendance",
                "log",
                "--exclusion",
                "al",
                "--date",
                "2025-12-03",
                "--half-day",
                "--description",
                "Monza",
                "--append",
            ])
            .unwrap();
            let expected = LogArgs::builder()
                .exclusion(Exclusion::AnnualLeave)
                .date(NaiveDate::from_ymd_opt(2025, 12, 3).unwrap())
                .half_day(true)
                .description("Monza".into())
                .flags(LogFlags {
                    append: true,
                    delete: false,
                })
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }
    }

    mod show_tests {
        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "show"]).unwrap();
            let expected = ShowArgs { top: Option::None };
            assert_eq!(args.command(), &Command::Show(expected))
        }

        #[test]
        fn parses_with_top() {
            let args = Cli::try_parse_from(&["attendance", "show", "--top", "10"]).unwrap();
            let expected = ShowArgs {
                top: Option::Some(10),
            };
            assert_eq!(args.command(), &Command::Show(expected))
        }

        #[test]
        fn returns_error_if_top_not_an_int() {
            let result = Cli::try_parse_from(&["attendance", "show", "--top", "cat"]);
            assert!(result.is_err())
        }
    }

    mod summary_tests {
        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "summary"]).unwrap();
            let expected = SummaryArgs {
                months: Option::None,
            };
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn parses_with_months() {
            let args = Cli::try_parse_from(&["attendance", "summary", "--months", "3"]).unwrap();
            let expected = SummaryArgs {
                months: Option::Some(3),
            };
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn returns_error_if_months_not_an_int() {
            let result = Cli::try_parse_from(&["attendance", "summary", "--months", "ago"]);
            assert!(result.is_err())
        }
    }
}
