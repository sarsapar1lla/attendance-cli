use clap::{Parser, Subcommand};

pub mod log;
pub mod show;
pub mod summary;

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
    Log(log::Arguments),

    /// Show log
    Show(show::Arguments),

    /// Summarise attendance
    Summary(summary::Arguments),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod log_tests {
        use chrono::NaiveDate;

        use crate::cli::log::{Arguments, Exclusion, Mode};

        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "log"]).unwrap();
            let expected = Arguments::builder()
                .half_day(false)
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        mod exclusion_tests {
            use super::*;

            #[test]
            fn parses_with_exclusion() {
                let args =
                    Cli::try_parse_from(&["attendance", "log", "--exclusion", "wfh"]).unwrap();
                let expected = Arguments::builder()
                    .exclusion(Exclusion::WorkingFromHome)
                    .half_day(false)
                    .mode(Mode::Create)
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
                let expected = Arguments::builder()
                    .date(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap())
                    .half_day(false)
                    .mode(Mode::Create)
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
            let expected = Arguments::builder()
                .half_day(true)
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_description() {
            let args =
                Cli::try_parse_from(&["attendance", "log", "--description", "Party!"]).unwrap();
            let expected = Arguments::builder()
                .description("Party!".into())
                .half_day(false)
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_create() {
            let args = Cli::try_parse_from(&["attendance", "log", "--mode", "create"]).unwrap();
            let expected = Arguments::builder()
                .half_day(false)
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_append() {
            let args = Cli::try_parse_from(&["attendance", "log", "--mode", "append"]).unwrap();
            let expected = Arguments::builder()
                .half_day(false)
                .mode(Mode::Append)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_delete() {
            let args = Cli::try_parse_from(&["attendance", "log", "--mode", "delete"]).unwrap();
            let expected = Arguments::builder()
                .half_day(false)
                .mode(Mode::Delete)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
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
                "--mode",
                "append",
            ])
            .unwrap();
            let expected = Arguments::builder()
                .exclusion(Exclusion::AnnualLeave)
                .date(NaiveDate::from_ymd_opt(2025, 12, 3).unwrap())
                .half_day(true)
                .description("Monza".into())
                .mode(Mode::Append)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }
    }

    mod show_tests {
        use crate::cli::show::Arguments;

        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "show"]).unwrap();
            let expected = Arguments::builder().build();
            assert_eq!(args.command(), &Command::Show(expected))
        }

        #[test]
        fn parses_with_top() {
            let args = Cli::try_parse_from(&["attendance", "show", "--top", "10"]).unwrap();
            let expected = Arguments::builder().top(10).build();
            assert_eq!(args.command(), &Command::Show(expected))
        }

        #[test]
        fn returns_error_if_top_not_an_int() {
            let result = Cli::try_parse_from(&["attendance", "show", "--top", "cat"]);
            assert!(result.is_err())
        }
    }

    mod summary_tests {
        use crate::cli::summary::Arguments;

        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::try_parse_from(&["attendance", "summary"]).unwrap();
            let expected = Arguments::builder().build();
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn parses_with_months() {
            let args = Cli::try_parse_from(&["attendance", "summary", "--months", "3"]).unwrap();
            let expected = Arguments::builder().months(3).build();
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn returns_error_if_months_not_an_int() {
            let result = Cli::try_parse_from(&["attendance", "summary", "--months", "ago"]);
            assert!(result.is_err())
        }
    }
}
