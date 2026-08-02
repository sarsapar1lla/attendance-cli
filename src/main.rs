use std::io;

use clap::Parser;
use jiff::Zoned;

use crate::{
    cli::{Cli, Command},
    printer::{record, summary},
    repository::FileRepository,
};

mod category;
mod cli;
mod error;
mod handler;
mod model;
mod printer;
mod repository;

fn main() {
    let cli = Cli::parse();
    let repository = FileRepository::new().expect("Repository can be instantiated");

    init_logging(cli.debug());

    let result = match cli.command() {
        Command::Completion(args) => {
            handler::completion(args, &mut io::stdout());
            Ok(())
        }
        Command::Log(args) => handler::log(args, &repository),
        Command::Show(args) => handler::show(args, &repository, &record::Table),
        Command::Spec => {
            handler::spec(&mut io::stdout());
            Ok(())
        }
        Command::Summary(args) => {
            let printer = summary::from_args(args);
            handler::summary(args, &repository, printer.as_ref(), || {
                Zoned::now().datetime()
            })
        }
    };

    match result {
        Ok(()) => {}
        Err(error) => tracing::error!("{error}"),
    }
}

fn init_logging(debug: bool) {
    let level = if debug {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::WARN
    };
    let format = tracing_subscriber::fmt::format().pretty();
    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(level)
        .init();
}
