use std::io;

use chrono::Utc;
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    printer::{TableRecordPrinter, TableSummaryPrinter},
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
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let repository = FileRepository::new();

    let result = match cli.command() {
        Command::Completion(args) => handler::completion(args, &mut io::stdout()),
        Command::Log(args) => handler::log(args, &repository),
        Command::Show(args) => handler::show(args, &repository, &TableRecordPrinter),
        Command::Summary(args) => {
            handler::summary(args, &repository, &TableSummaryPrinter, Utc::now)
        }
    };

    match result {
        Ok(()) => {}
        Err(error) => tracing::error!("{error}"),
    }
}
