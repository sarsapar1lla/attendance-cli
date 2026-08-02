use clap::{ArgAction, Parser, Subcommand};

pub mod completion;
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

    #[arg(long, action = ArgAction::SetTrue, hide = true, global = true)]
    debug: bool,
}

impl Cli {
    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}

#[derive(Debug, Subcommand)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Command {
    /// Generate completions
    Completion(completion::Arguments),

    /// Log attendance
    Log(log::Arguments),

    /// Show log
    Show(show::Arguments),

    /// Output Cli spec
    #[command(hide = true)]
    Spec,

    /// Summarise attendance
    Summary(summary::Arguments),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod debug_tests {
        use super::*;

        #[test]
        fn parses_with_default() {
            let args = Cli::parse_from(&["attendance", "show"]);
            assert_eq!(args.debug(), false)
        }

        #[test]
        fn parses_with_debug() {
            let args = Cli::parse_from(&["attendance", "--debug", "show"]);
            assert_eq!(args.debug(), true)
        }
    }

    mod completion_tests {
        use crate::cli::completion::{Arguments, Shell};

        use super::*;

        #[test]
        fn parses_with_bash() {
            let args = Cli::parse_from(&["attendance", "completion", "bash"]);
            let expected = Arguments::new(Shell::Bash);
            assert_eq!(args.command(), &Command::Completion(expected))
        }

        #[test]
        fn parses_with_fish() {
            let args = Cli::parse_from(&["attendance", "completion", "fish"]);
            let expected = Arguments::new(Shell::Fish);
            assert_eq!(args.command(), &Command::Completion(expected))
        }

        #[test]
        fn parses_with_zsh() {
            let args = Cli::parse_from(&["attendance", "completion", "zsh"]);
            let expected = Arguments::new(Shell::Zsh);
            assert_eq!(args.command(), &Command::Completion(expected))
        }

        #[test]
        fn parses_with_debug() {
            let args = Cli::parse_from(&["attendance", "completion", "zsh", "--debug"]);
            assert_eq!(args.debug(), true)
        }

        #[test]
        fn returns_error_if_invalid_shell() {
            let args = Cli::try_parse_from(&["attendance", "completion", "nushell"]);
            assert!(args.is_err())
        }
    }

    mod log_tests {

        use crate::cli::log::{Arguments, HalfDay, Mode, RecordType};

        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::parse_from(&["attendance", "log"]);
            let expected = Arguments::builder()
                .record_type(RecordType::Office)
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        #[test]
        fn parses_with_debug() {
            let args = Cli::parse_from(&["attendance", "log", "--debug"]);
            assert_eq!(args.debug(), true)
        }

        mod record_type_tests {
            use super::*;

            #[test]
            fn parses_with_default() {
                let args = Cli::parse_from(&["attendance", "log"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .mode(Mode::Create)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn parses_with_record_type() {
                let args = Cli::parse_from(&["attendance", "log", "--type", "wfh"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::WorkingFromHome)
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
                let args = Cli::parse_from(&["attendance", "log", "--date", "2025-12-01"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .date(jiff::civil::date(2025, 12, 1))
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

        mod half_day_tests {
            use super::*;

            #[test]
            fn parses_with_am() {
                let args = Cli::parse_from(&["attendance", "log", "--half-day", "am"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .half_day(HalfDay::Am)
                    .mode(Mode::Create)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn parses_with_pm() {
                let args = Cli::parse_from(&["attendance", "log", "--half-day", "pm"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .half_day(HalfDay::Pm)
                    .mode(Mode::Create)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn returns_error_if_not_valid_half_day() {
                let result = Cli::try_parse_from(&["attendance", "log", "--half-day", "invalid"]);
                assert!(result.is_err())
            }
        }

        #[test]
        fn parses_with_description() {
            let args = Cli::parse_from(&["attendance", "log", "--description", "Party!"]);
            let expected = Arguments::builder()
                .record_type(RecordType::Office)
                .description("Party!".into())
                .mode(Mode::Create)
                .build();
            assert_eq!(args.command(), &Command::Log(expected))
        }

        mod mode_tests {
            use super::*;

            #[test]
            fn parses_with_default() {
                let args = Cli::parse_from(&["attendance", "log"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .mode(Mode::Create)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn parses_with_create() {
                let args = Cli::parse_from(&["attendance", "log", "--mode", "create"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .mode(Mode::Create)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn parses_with_append() {
                let args = Cli::parse_from(&["attendance", "log", "--mode", "append"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .mode(Mode::Append)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }

            #[test]
            fn parses_with_delete() {
                let args = Cli::parse_from(&["attendance", "log", "--mode", "delete"]);
                let expected = Arguments::builder()
                    .record_type(RecordType::Office)
                    .mode(Mode::Delete)
                    .build();
                assert_eq!(args.command(), &Command::Log(expected))
            }
        }

        #[test]
        fn parses_with_everything() {
            let args = Cli::parse_from(&[
                "attendance",
                "log",
                "--type",
                "al",
                "--date",
                "2025-12-03",
                "--half-day",
                "am",
                "--description",
                "Monza",
                "--mode",
                "append",
            ]);
            let expected = Arguments::builder()
                .record_type(RecordType::AnnualLeave)
                .date(jiff::civil::date(2025, 12, 3))
                .half_day(HalfDay::Am)
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
        fn parses_with_default() {
            let args = Cli::parse_from(&["attendance", "show"]);
            let expected = Arguments::builder().top(10).build();
            assert_eq!(args.command(), &Command::Show(expected))
        }

        #[test]
        fn parses_with_debug() {
            let args = Cli::parse_from(&["attendance", "show", "--debug"]);
            assert_eq!(args.debug(), true)
        }

        mod top_tests {
            use super::*;

            #[test]
            fn parses_with_top() {
                let args = Cli::parse_from(&["attendance", "show", "--top", "5"]);
                let expected = Arguments::builder().top(5).build();
                assert_eq!(args.command(), &Command::Show(expected))
            }

            #[test]
            fn returns_error_if_top_not_an_int() {
                let result = Cli::try_parse_from(&["attendance", "show", "--top", "cat"]);
                assert!(result.is_err())
            }
        }

        mod date_tests {

            use super::*;

            #[test]
            fn parses_with_date() {
                let args = Cli::parse_from(&["attendance", "show", "--date", "2025-12-01"]);
                let expected = Arguments::builder()
                    .top(10)
                    .date(jiff::civil::date(2025, 12, 1))
                    .build();
                assert_eq!(args.command(), &Command::Show(expected))
            }

            #[test]
            fn returns_error_if_date_not_valid() {
                let result = Cli::try_parse_from(&["attendance", "show", "--date", "fish"]);
                assert!(result.is_err())
            }
        }
    }

    mod spec_tests {
        use super::*;

        #[test]
        fn parses() {
            let args = Cli::parse_from(&["attendance", "spec"]);
            assert_eq!(args.command(), &Command::Spec)
        }
    }

    mod summary_tests {
        use crate::cli::summary::Arguments;

        use super::*;

        #[test]
        fn parses_with_no_args() {
            let args = Cli::parse_from(&["attendance", "summary"]);
            let expected = Arguments::builder().json(false).build();
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn parses_with_debug() {
            let args = Cli::parse_from(&["attendance", "summary", "--debug"]);
            assert_eq!(args.debug(), true)
        }

        #[test]
        fn parses_with_months() {
            let args = Cli::parse_from(&["attendance", "summary", "--months", "3"]);
            let expected = Arguments::builder().months(3).json(false).build();
            assert_eq!(args.command(), &Command::Summary(expected))
        }

        #[test]
        fn returns_error_if_months_not_an_int() {
            let result = Cli::try_parse_from(&["attendance", "summary", "--months", "ago"]);
            assert!(result.is_err())
        }

        #[test]
        fn parses_with_json() {
            let args = Cli::parse_from(&["attendance", "summary", "--json"]);
            let expected = Arguments::builder().json(true).build();
            assert_eq!(args.command(), &Command::Summary(expected))
        }
    }
}
